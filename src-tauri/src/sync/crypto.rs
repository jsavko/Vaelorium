//! E2EE primitives for sync.
//!
//! - **KDF:** Argon2id (OWASP-recommended defaults; salt stored unencrypted
//!   per Tome alongside the encrypted blobs so any device with the passphrase
//!   can derive the same key).
//! - **AEAD:** ChaCha20-Poly1305 with a fresh random 12-byte nonce per message.
//!
//! Wire format for an encrypted blob: `[nonce(12)] || [ciphertext+tag]`.
//! The salt is stored separately in the per-Tome `meta.enc` header (and locally
//! in `sync_config.passphrase_salt`) — not embedded in every message — so a
//! single passphrase derives one key per Tome.
//!
//! This module deliberately avoids `age` because age's passphrase mode uses
//! scrypt internally; mixing it with Argon2id-via-other-crate is awkward.
//! ChaCha20-Poly1305 + Argon2id is the modern, recommended primitive stack.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce};
use rand::RngCore;
use thiserror::Error;
use zeroize::Zeroize;

pub const SALT_LEN: usize = 16;
pub const NONCE_LEN: usize = 12;
pub const KEY_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("argon2 error: {0}")]
    Kdf(String),

    #[error("encryption failed")]
    Encrypt,

    #[error("decryption failed (wrong passphrase or tampered ciphertext)")]
    Decrypt,

    #[error("ciphertext too short to contain a nonce")]
    Malformed,
}

/// 32-byte symmetric key derived from a passphrase + per-Tome salt.
///
/// Holds raw key bytes; zeroed on drop to limit residency in memory.
pub struct KeyMaterial {
    bytes: [u8; KEY_LEN],
}

impl KeyMaterial {
    /// Derive a [`KeyMaterial`] from a passphrase using Argon2id with strong
    /// defaults (`m=64MiB, t=3, p=1` — OWASP-recommended for interactive use).
    ///
    /// The same `(passphrase, salt)` always derives the same key.
    pub fn derive(passphrase: &str, salt: &[u8]) -> Result<Self, CryptoError> {
        // OWASP 2024 recommendation for Argon2id interactive: m=64MiB, t=3, p=1.
        let params = Params::new(64 * 1024, 3, 1, Some(KEY_LEN))
            .map_err(|e| CryptoError::Kdf(e.to_string()))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

        let mut bytes = [0u8; KEY_LEN];
        argon
            .hash_password_into(passphrase.as_bytes(), salt, &mut bytes)
            .map_err(|e| CryptoError::Kdf(e.to_string()))?;
        Ok(Self { bytes })
    }

    /// Borrow the raw key bytes (e.g. for the AEAD constructor).
    pub fn as_bytes(&self) -> &[u8; KEY_LEN] {
        &self.bytes
    }
}

impl Drop for KeyMaterial {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

/// Generate a fresh cryptographically-random salt for a new Tome's KDF setup.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    salt
}

/// Encrypt `plaintext` with `key`. Output format: `[nonce(12)] || [ciphertext+tag]`.
pub fn encrypt(key: &KeyMaterial, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::Encrypt)?;

    let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

/// Decrypt a `[nonce] || [ciphertext+tag]` blob with `key`.
pub fn decrypt(key: &KeyMaterial, blob: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if blob.len() < NONCE_LEN {
        return Err(CryptoError::Malformed);
    }
    let (nonce_bytes, ciphertext) = blob.split_at(NONCE_LEN);
    let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_bytes()));
    let nonce = Nonce::from_slice(nonce_bytes);
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CryptoError::Decrypt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_derivation_is_deterministic() {
        let salt = generate_salt();
        let k1 = KeyMaterial::derive("correct horse battery staple", &salt).unwrap();
        let k2 = KeyMaterial::derive("correct horse battery staple", &salt).unwrap();
        assert_eq!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn different_passphrases_yield_different_keys() {
        let salt = generate_salt();
        let k1 = KeyMaterial::derive("passphrase one", &salt).unwrap();
        let k2 = KeyMaterial::derive("passphrase two", &salt).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn different_salts_yield_different_keys() {
        let s1 = generate_salt();
        let s2 = generate_salt();
        assert_ne!(s1, s2);
        let k1 = KeyMaterial::derive("same passphrase", &s1).unwrap();
        let k2 = KeyMaterial::derive("same passphrase", &s2).unwrap();
        assert_ne!(k1.as_bytes(), k2.as_bytes());
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let salt = generate_salt();
        let key = KeyMaterial::derive("test passphrase", &salt).unwrap();
        let plaintext = b"the quick brown fox jumps over the lazy dog";

        let blob = encrypt(&key, plaintext).unwrap();
        assert_ne!(&blob[NONCE_LEN..], plaintext); // not plaintext-equivalent
        assert!(blob.len() > plaintext.len()); // nonce + tag overhead

        let decrypted = decrypt(&key, &blob).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_passphrase_fails_decryption() {
        let salt = generate_salt();
        let k1 = KeyMaterial::derive("correct", &salt).unwrap();
        let k2 = KeyMaterial::derive("wrong", &salt).unwrap();
        let blob = encrypt(&k1, b"secret").unwrap();
        let result = decrypt(&k2, &blob);
        assert!(matches!(result, Err(CryptoError::Decrypt)));
    }

    #[test]
    fn tampered_ciphertext_fails_decryption() {
        let salt = generate_salt();
        let key = KeyMaterial::derive("p", &salt).unwrap();
        let mut blob = encrypt(&key, b"secret").unwrap();
        // Flip a bit somewhere in the ciphertext (after the nonce).
        blob[NONCE_LEN + 1] ^= 0x01;
        let result = decrypt(&key, &blob);
        assert!(matches!(result, Err(CryptoError::Decrypt)));
    }

    #[test]
    fn malformed_short_blob_rejected() {
        let salt = generate_salt();
        let key = KeyMaterial::derive("p", &salt).unwrap();
        let result = decrypt(&key, b"short");
        assert!(matches!(result, Err(CryptoError::Malformed)));
    }

    #[test]
    fn each_encryption_uses_fresh_nonce() {
        let salt = generate_salt();
        let key = KeyMaterial::derive("p", &salt).unwrap();
        let blob1 = encrypt(&key, b"same plaintext").unwrap();
        let blob2 = encrypt(&key, b"same plaintext").unwrap();
        // Same plaintext + same key should produce different ciphertext
        // because the nonce is freshly generated each time.
        assert_ne!(blob1, blob2);
        // Both still decrypt back to the original.
        assert_eq!(decrypt(&key, &blob1).unwrap(), b"same plaintext");
        assert_eq!(decrypt(&key, &blob2).unwrap(), b"same plaintext");
    }
}
