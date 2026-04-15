# src/encryption/

Cryptographic primitives for protecting financial data at rest. Uses AES-256-GCM for authenticated encryption and Argon2id plus HKDF-SHA256 for password-based key derivation.

## Files

### mod.rs
Module root. Documents the key derivation flow (password to Argon2id to HKDF to purpose-specific keys) and re-exports the public types.

### key.rs
Password-based key derivation. `DerivedKey` wraps a 256-bit symmetric key produced by Argon2id with OWASP-recommended parameters (64 MB memory, 3 iterations, 4 parallelism). HKDF-SHA256 derives domain-separated keys for "database", "config", and "backup" purposes from the master key.

### cipher.rs
AES-256-GCM operations. `encrypt` generates a 12-byte random nonce and returns a ciphertext blob with the authentication tag appended. `decrypt` verifies the tag before returning plaintext. Each call uses a fresh nonce to prevent reuse.

### secure_memory.rs
Zeroizing wrapper types. `SecureString` and `SecureBytes` wrap `String`/`Vec<u8>` in `zeroize::Zeroizing` so sensitive buffers are wiped from memory when dropped. Both implement `Deref`/`DerefMut` for transparent use.
