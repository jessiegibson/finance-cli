//! Encryption module for the Finance CLI.
//!
//! This module provides comprehensive cryptographic functionality for protecting
//! financial data at rest. All sensitive data is encrypted using AES-256-GCM
//! with keys derived from user passwords via Argon2id.
//!
//! # Architecture
//!
//! ```text
//! User Password/Recovery Phrase
//!         │
//!         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                  Argon2id Key Derivation                     │
//! │  Memory: 64MB, Iterations: 3, Parallelism: 4               │
//! └─────────────────────────────────────────────────────────────┘
//!         │
//!         ▼
//! ┌─────────────────────────────────────────────────────────────┐
//! │                 Master Key (256-bit)                        │
//! │                 (Memory only, never stored)                 │
//! └─────────────────────────────────────────────────────────────┘
//!         │
//!         ├── HKDF-SHA256("database") ──► Database Encryption Key
//!         ├── HKDF-SHA256("config")   ──► Config Encryption Key
//!         └── HKDF-SHA256("backup")   ──► Backup Encryption Key
//! ```
//!
//! # Security Properties
//!
//! - **Confidentiality**: AES-256 encryption protects data contents
//! - **Integrity**: GCM authentication detects tampering
//! - **Forward Secrecy**: Keys zeroized immediately after use
//! - **Memory Safety**: All sensitive data uses secure memory types

pub mod cipher;
pub mod key;
pub mod secure_memory;

pub use cipher::{decrypt, encrypt};
pub use key::{derive_key, DerivedKey, KeyDomain, Salt};
pub use secure_memory::{SecureBytes, SecureString};

use crate::error::{EncryptionError, Error, Result};

/// Current encryption format version.
pub const ENCRYPTION_VERSION: u8 = 1;

/// Magic bytes for encrypted file identification.
pub const MAGIC_BYTES: &[u8; 8] = b"FINCRYPT";

/// Initialize the encryption subsystem.
pub fn init() -> Result<()> {
    // Verify that we can generate random bytes
    use rand::RngCore;
    let mut test_bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut test_bytes);

    // Verify all bytes are not zero (extremely unlikely if RNG works)
    if test_bytes.iter().all(|&b| b == 0) {
        return Err(Error::Encryption(EncryptionError::KeyDerivationFailed(
            "Random number generator appears non-functional".into(),
        )));
    }

    tracing::info!("Encryption subsystem initialized successfully");
    Ok(())
}

/// Cleanup the encryption subsystem.
pub fn cleanup() {
    tracing::info!("Encryption subsystem cleanup completed");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_and_cleanup() {
        init().expect("Encryption init should succeed");
        cleanup();
    }

    #[test]
    fn test_magic_bytes_constant() {
        assert_eq!(MAGIC_BYTES, b"FINCRYPT");
        assert_eq!(MAGIC_BYTES.len(), 8);
    }
}
