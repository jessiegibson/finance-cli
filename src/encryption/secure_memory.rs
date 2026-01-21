//! Secure memory handling with automatic zeroization.
//!
//! This module provides secure memory types that automatically zeroize their
//! contents when dropped, preventing sensitive data from remaining in memory
//! after use.

use std::ops::{Deref, DerefMut};
use zeroize::{Zeroize, Zeroizing};

/// Secure string that automatically zeroizes on drop.
///
/// This type wraps a `String` and ensures it is zeroized when no longer needed.
/// Use this for passwords, passphrases, and other sensitive text data.
#[derive(Clone)]
pub struct SecureString(Zeroizing<String>);

impl SecureString {
    /// Create a new secure string.
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    /// Create an empty secure string.
    pub fn empty() -> Self {
        Self::new(String::new())
    }

    /// Get the length of the string.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the string is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Get the inner string as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for SecureString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureString([REDACTED])")
    }
}

impl From<String> for SecureString {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for SecureString {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

/// Secure byte buffer that automatically zeroizes on drop.
///
/// Use this for encryption keys, derived keys, and other sensitive binary data.
#[derive(Clone)]
pub struct SecureBytes(Zeroizing<Vec<u8>>);

impl SecureBytes {
    /// Create a new secure byte buffer.
    pub fn new(value: Vec<u8>) -> Self {
        Self(Zeroizing::new(value))
    }

    /// Create a secure buffer of the specified size filled with zeros.
    pub fn zeros(len: usize) -> Self {
        Self::new(vec![0u8; len])
    }

    /// Create a secure buffer with random bytes.
    pub fn random(len: usize) -> Self {
        use rand::RngCore;
        let mut bytes = vec![0u8; len];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self::new(bytes)
    }

    /// Get the length of the buffer.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Deref for SecureBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SecureBytes {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::fmt::Debug for SecureBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SecureBytes([REDACTED, {} bytes])", self.0.len())
    }
}

impl From<Vec<u8>> for SecureBytes {
    fn from(value: Vec<u8>) -> Self {
        Self::new(value)
    }
}

impl From<&[u8]> for SecureBytes {
    fn from(value: &[u8]) -> Self {
        Self::new(value.to_vec())
    }
}

impl AsRef<[u8]> for SecureBytes {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_string_redacts_debug() {
        let s = SecureString::new("secret".to_string());
        let debug = format!("{:?}", s);
        assert!(!debug.contains("secret"));
        assert!(debug.contains("REDACTED"));
    }

    #[test]
    fn test_secure_bytes_random() {
        let b1 = SecureBytes::random(32);
        let b2 = SecureBytes::random(32);
        assert_ne!(&*b1, &*b2);
        assert_eq!(b1.len(), 32);
    }

    #[test]
    fn test_secure_string_operations() {
        let s = SecureString::new("hello".to_string());
        assert_eq!(s.len(), 5);
        assert!(!s.is_empty());
        assert_eq!(&*s, "hello");
    }
}
