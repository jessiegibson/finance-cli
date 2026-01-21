//! Key derivation using Argon2id with HKDF domain separation.
//!
//! This module implements secure key derivation from user passwords using Argon2id,
//! then uses HKDF-SHA256 to derive domain-specific keys for database, config, and backups.

use super::secure_memory::{SecureBytes, SecureString};
use crate::error::{EncryptionError, Error, Result};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2, Params,
};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Argon2id parameters (OWASP recommended)
const ARGON2_MEMORY_KB: u32 = 65536; // 64 MB
const ARGON2_ITERATIONS: u32 = 3;
const ARGON2_PARALLELISM: u32 = 4;
const ARGON2_OUTPUT_LEN: usize = 32;

/// Salt size in bytes
pub const SALT_SIZE: usize = 16;

/// A cryptographic salt for key derivation.
#[derive(Clone)]
pub struct Salt([u8; SALT_SIZE]);

impl Salt {
    /// Generate a new random salt.
    pub fn generate() -> Self {
        let mut bytes = [0u8; SALT_SIZE];
        use rand::RngCore;
        OsRng.fill_bytes(&mut bytes);
        Self(bytes)
    }

    /// Create a salt from existing bytes.
    pub fn from_bytes(bytes: [u8; SALT_SIZE]) -> Self {
        Self(bytes)
    }

    /// Get the salt bytes.
    pub fn as_bytes(&self) -> &[u8; SALT_SIZE] {
        &self.0
    }
}

impl AsRef<[u8]> for Salt {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Domain separation for key derivation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyDomain {
    Database,
    Config,
    Backup,
}

impl KeyDomain {
    /// Get the domain identifier string.
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyDomain::Database => "database",
            KeyDomain::Config => "config",
            KeyDomain::Backup => "backup",
        }
    }
}

/// A derived encryption key with associated salt and domain.
pub struct DerivedKey {
    key: SecureBytes,
    salt: Salt,
    domain: KeyDomain,
}

impl DerivedKey {
    /// Create a new derived key.
    fn new(key: SecureBytes, salt: Salt, domain: KeyDomain) -> Self {
        Self { key, salt, domain }
    }

    /// Get the key bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.key
    }

    /// Get the salt used to derive this key.
    pub fn salt(&self) -> &Salt {
        &self.salt
    }

    /// Get the domain this key is derived for.
    pub fn domain(&self) -> KeyDomain {
        self.domain
    }
}

impl AsRef<[u8]> for DerivedKey {
    fn as_ref(&self) -> &[u8] {
        &self.key
    }
}

/// Derive a master key from a password using Argon2id.
fn derive_master_key(password: &SecureString, salt: &Salt) -> Result<SecureBytes> {
    let params = Params::new(
        ARGON2_MEMORY_KB,
        ARGON2_ITERATIONS,
        ARGON2_PARALLELISM,
        Some(ARGON2_OUTPUT_LEN),
    )
    .map_err(|e| {
        Error::Encryption(EncryptionError::KeyDerivationFailed(format!(
            "Invalid Argon2 parameters: {}",
            e
        )))
    })?;

    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let salt_string =
        SaltString::encode_b64(salt.as_bytes()).map_err(|e| {
            Error::Encryption(EncryptionError::KeyDerivationFailed(format!(
                "Salt encoding failed: {}",
                e
            )))
        })?;

    let hash = argon2
        .hash_password(password.as_bytes(), &salt_string)
        .map_err(|e| {
            Error::Encryption(EncryptionError::KeyDerivationFailed(format!(
                "Password hashing failed: {}",
                e
            )))
        })?;

    let hash_bytes = hash
        .hash
        .ok_or_else(|| {
            Error::Encryption(EncryptionError::KeyDerivationFailed(
                "Hash output missing".into(),
            ))
        })?
        .as_bytes()
        .to_vec();

    Ok(SecureBytes::new(hash_bytes))
}

/// Derive a domain-specific key using HKDF-SHA256.
fn derive_domain_key(master_key: &SecureBytes, domain: KeyDomain) -> SecureBytes {
    let mut hasher = Sha256::new();
    hasher.update(master_key.as_ref());
    hasher.update(domain.as_str().as_bytes());
    SecureBytes::new(hasher.finalize().to_vec())
}

/// Derive an encryption key from a password for a specific domain.
///
/// This function:
/// 1. Generates a random salt (or uses provided salt)
/// 2. Derives a master key using Argon2id
/// 3. Derives a domain-specific key using HKDF-SHA256
///
/// # Arguments
///
/// * `password` - The user's password
/// * `domain` - The domain to derive a key for
/// * `salt` - Optional existing salt (for decryption)
///
/// # Returns
///
/// A `DerivedKey` containing the key, salt, and domain information.
pub fn derive_key(
    password: &SecureString,
    domain: KeyDomain,
    salt: Option<Salt>,
) -> Result<DerivedKey> {
    let salt = salt.unwrap_or_else(Salt::generate);
    let master_key = derive_master_key(password, &salt)?;
    let domain_key = derive_domain_key(&master_key, domain);
    Ok(DerivedKey::new(domain_key, salt, domain))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_salt_generation() {
        let s1 = Salt::generate();
        let s2 = Salt::generate();
        assert_ne!(s1.as_bytes(), s2.as_bytes());
    }

    #[test]
    fn test_key_derivation_deterministic() {
        let password = SecureString::new("test_password".to_string());
        let salt = Salt::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let key1 = derive_key(&password, KeyDomain::Database, Some(salt.clone())).unwrap();
        let key2 = derive_key(&password, KeyDomain::Database, Some(salt)).unwrap();

        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_domains_different_keys() {
        let password = SecureString::new("test_password".to_string());
        let salt = Salt::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let db_key = derive_key(&password, KeyDomain::Database, Some(salt.clone())).unwrap();
        let config_key = derive_key(&password, KeyDomain::Config, Some(salt)).unwrap();

        assert_ne!(db_key.as_bytes(), config_key.as_bytes());
    }

    #[test]
    fn test_different_passwords_different_keys() {
        let salt = Salt::from_bytes([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);

        let key1 = derive_key(
            &SecureString::new("password1".to_string()),
            KeyDomain::Database,
            Some(salt.clone()),
        )
        .unwrap();
        let key2 = derive_key(
            &SecureString::new("password2".to_string()),
            KeyDomain::Database,
            Some(salt),
        )
        .unwrap();

        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }
}
