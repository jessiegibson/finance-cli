//! AES-256-GCM encryption and decryption.
//!
//! This module provides authenticated encryption using AES-256-GCM.
//! Each encryption operation uses a unique random nonce.

use super::key::DerivedKey;
use super::secure_memory::SecureBytes;
use crate::error::{EncryptionError, Error, Result};
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

/// Nonce size for AES-GCM (96 bits = 12 bytes)
pub const NONCE_SIZE: usize = 12;

/// Authentication tag size for AES-GCM (128 bits = 16 bytes)
pub const TAG_SIZE: usize = 16;

/// Encrypt data using AES-256-GCM.
///
/// Returns the ciphertext with the nonce prepended.
/// Format: [nonce (12 bytes)][ciphertext + tag]
///
/// # Arguments
///
/// * `plaintext` - The data to encrypt
/// * `key` - The encryption key (must be 32 bytes)
///
/// # Returns
///
/// Encrypted data with nonce prepended.
pub fn encrypt(plaintext: &[u8], key: &DerivedKey) -> Result<Vec<u8>> {
    let key_bytes = key.as_bytes();
    if key_bytes.len() != 32 {
        return Err(Error::Encryption(EncryptionError::EncryptionFailed(
            "Key must be 32 bytes".into(),
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| {
        Error::Encryption(EncryptionError::EncryptionFailed(format!(
            "Failed to create cipher: {}",
            e
        )))
    })?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher.encrypt(nonce, plaintext).map_err(|e| {
        Error::Encryption(EncryptionError::EncryptionFailed(format!(
            "Encryption failed: {}",
            e
        )))
    })?;

    // Prepend nonce to ciphertext
    let mut result = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data using AES-256-GCM.
///
/// Expects data in the format: [nonce (12 bytes)][ciphertext + tag]
///
/// # Arguments
///
/// * `ciphertext` - The encrypted data with prepended nonce
/// * `key` - The decryption key (must be 32 bytes)
///
/// # Returns
///
/// The decrypted plaintext.
pub fn decrypt(ciphertext: &[u8], key: &DerivedKey) -> Result<SecureBytes> {
    if ciphertext.len() < NONCE_SIZE + TAG_SIZE {
        return Err(Error::Encryption(EncryptionError::DecryptionFailed(
            "Ciphertext too short".into(),
        )));
    }

    let key_bytes = key.as_bytes();
    if key_bytes.len() != 32 {
        return Err(Error::Encryption(EncryptionError::DecryptionFailed(
            "Key must be 32 bytes".into(),
        )));
    }

    let cipher = Aes256Gcm::new_from_slice(key_bytes).map_err(|e| {
        Error::Encryption(EncryptionError::DecryptionFailed(format!(
            "Failed to create cipher: {}",
            e
        )))
    })?;

    // Extract nonce and ciphertext
    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);
    let nonce = Nonce::from_slice(nonce_bytes);

    // Decrypt
    let plaintext = cipher.decrypt(nonce, encrypted).map_err(|_| {
        Error::Encryption(EncryptionError::DecryptionFailed(
            "Decryption failed - invalid key or corrupted data".into(),
        ))
    })?;

    Ok(SecureBytes::new(plaintext))
}

/// Encrypt a string to base64-encoded ciphertext.
pub fn encrypt_string(plaintext: &str, key: &DerivedKey) -> Result<String> {
    let ciphertext = encrypt(plaintext.as_bytes(), key)?;
    Ok(base64::Engine::encode(
        &base64::engine::general_purpose::STANDARD,
        &ciphertext,
    ))
}

/// Decrypt base64-encoded ciphertext to a string.
pub fn decrypt_string(ciphertext_b64: &str, key: &DerivedKey) -> Result<String> {
    let ciphertext = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        ciphertext_b64,
    )
    .map_err(|e| {
        Error::Encryption(EncryptionError::DecryptionFailed(format!(
            "Invalid base64: {}",
            e
        )))
    })?;

    let plaintext = decrypt(&ciphertext, key)?;
    String::from_utf8(plaintext.to_vec()).map_err(|e| {
        Error::Encryption(EncryptionError::DecryptionFailed(format!(
            "Invalid UTF-8: {}",
            e
        )))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::key::{derive_key, KeyDomain, Salt};
    use crate::encryption::secure_memory::SecureString;

    fn test_key() -> DerivedKey {
        let password = SecureString::new("test_password".to_string());
        let salt = Salt::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        derive_key(&password, KeyDomain::Database, Some(salt)).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"Hello, World! This is a test message.";

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(&*decrypted, plaintext);
    }

    #[test]
    fn test_different_nonces_each_encryption() {
        let key = test_key();
        let plaintext = b"Same message";

        let c1 = encrypt(plaintext, &key).unwrap();
        let c2 = encrypt(plaintext, &key).unwrap();

        // Nonces should be different (first 12 bytes)
        assert_ne!(&c1[..NONCE_SIZE], &c2[..NONCE_SIZE]);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"Sensitive data";

        let mut ciphertext = encrypt(plaintext, &key).unwrap();

        // Tamper with ciphertext
        let last_idx = ciphertext.len() - 1;
        ciphertext[last_idx] ^= 0xFF;

        let result = decrypt(&ciphertext, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = test_key();
        let password2 = SecureString::new("different_password".to_string());
        let salt = Salt::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let key2 = derive_key(&password2, KeyDomain::Database, Some(salt)).unwrap();

        let plaintext = b"Secret message";
        let ciphertext = encrypt(plaintext, &key1).unwrap();

        let result = decrypt(&ciphertext, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_string_encryption() {
        let key = test_key();
        let plaintext = "Hello, encrypted world!";

        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();

        assert_eq!(decrypted, plaintext);
    }
}
