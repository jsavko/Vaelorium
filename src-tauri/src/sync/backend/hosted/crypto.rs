//! Account-level crypto for Vaelorium Cloud signin / signout.
//!
//! Cloud is zero-knowledge: the server never sees the user's password.
//! The client runs Argon2id twice (same password, different context
//! strings concatenated to a per-account salt) to produce:
//!
//! - `auth_hash` — sent to the server, used to prove the user knows the
//!   password without revealing it.
//! - `enc_key` — kept locally, used to unwrap the server-stored
//!   `account_key` ciphertext (XChaCha20-Poly1305 AEAD).
//!
//! A third `recovery_key` context exists for M6+ recovery-phrase flows.
//!
//! All parameters must match the cloud side exactly
//! (`~/Projects/vaelorium-cloud/apps/api/src/auth/core.ts`) — any drift
//! produces a `invalid_credentials` 401 with no diagnostic.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::aead::Aead;
use chacha20poly1305::{KeyInit, XChaCha20Poly1305, XNonce};

/// Argon2id parameters for account-level KDF. **Frozen.** Any change
/// here requires coordinated cloud + app protocol version bump.
pub const ARGON2_MEM_KIB: u32 = 65_536; // 64 MiB
pub const ARGON2_TIME_COST: u32 = 3;
pub const ARGON2_PARALLELISM: u32 = 1;
pub const KDF_OUTPUT_LEN: usize = 32;

/// Wire format for XChaCha20-Poly1305 wraps: `nonce (24) || ciphertext || tag (16)`.
pub const XCHACHA_NONCE_LEN: usize = 24;

#[derive(Debug, thiserror::Error)]
pub enum CloudCryptoError {
    #[error("argon2 hash failure: {0}")]
    Argon2(String),

    #[error("wrapped account_key too short: {0} bytes (need at least {} + tag)", XCHACHA_NONCE_LEN)]
    WrappedTooShort(usize),

    #[error("account_key wrap failed to decrypt — wrong password or corrupted")]
    UnwrapFailed,

    #[error("account_key unwrap produced {0} bytes, expected 32")]
    WrongUnwrapLen(usize),
}

fn argon2_raw(password: &[u8], salt: &[u8]) -> Result<[u8; KDF_OUTPUT_LEN], CloudCryptoError> {
    let params = Params::new(ARGON2_MEM_KIB, ARGON2_TIME_COST, ARGON2_PARALLELISM, Some(KDF_OUTPUT_LEN))
        .map_err(|e| CloudCryptoError::Argon2(e.to_string()))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut out = [0u8; KDF_OUTPUT_LEN];
    argon2
        .hash_password_into(password, salt, &mut out)
        .map_err(|e| CloudCryptoError::Argon2(e.to_string()))?;
    Ok(out)
}

/// Build the context-suffixed salt the cloud protocol expects.
/// Format is raw salt bytes directly concatenated with the UTF-8 context
/// string — no separator, no null byte, no length prefix.
fn ctx_salt(kdf_salt_bytes: &[u8], context: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(kdf_salt_bytes.len() + context.len());
    out.extend_from_slice(kdf_salt_bytes);
    out.extend_from_slice(context.as_bytes());
    out
}

/// Argon2id(password, kdf_salt || "auth"). Base64 this and POST it to
/// `/api/auth/signin` as `auth_hash_b64`.
pub fn derive_auth_hash(password: &str, kdf_salt_bytes: &[u8]) -> Result<[u8; KDF_OUTPUT_LEN], CloudCryptoError> {
    argon2_raw(password.as_bytes(), &ctx_salt(kdf_salt_bytes, "auth"))
}

/// Argon2id(password, kdf_salt || "crypt"). Used locally to unwrap the
/// account_key ciphertext returned in the signin response. Never sent
/// to the server.
pub fn derive_enc_key(password: &str, kdf_salt_bytes: &[u8]) -> Result<[u8; KDF_OUTPUT_LEN], CloudCryptoError> {
    argon2_raw(password.as_bytes(), &ctx_salt(kdf_salt_bytes, "crypt"))
}

/// Argon2id(phrase, kdf_salt || "recov"). Reserved for recovery-phrase
/// password-reset flow (M6+). Exposed now so the shape is locked in.
#[allow(dead_code)]
pub fn derive_recovery_key(phrase: &str, kdf_salt_bytes: &[u8]) -> Result<[u8; KDF_OUTPUT_LEN], CloudCryptoError> {
    argon2_raw(phrase.as_bytes(), &ctx_salt(kdf_salt_bytes, "recov"))
}

/// Unwrap an XChaCha20-Poly1305 account_key envelope of the form
/// `nonce (24 bytes) || ciphertext || tag (16 bytes)`.
pub fn unwrap_account_key(
    wrapped: &[u8],
    enc_key: &[u8; KDF_OUTPUT_LEN],
) -> Result<[u8; KDF_OUTPUT_LEN], CloudCryptoError> {
    if wrapped.len() < XCHACHA_NONCE_LEN + 16 {
        return Err(CloudCryptoError::WrappedTooShort(wrapped.len()));
    }
    let (nonce_bytes, ciphertext) = wrapped.split_at(XCHACHA_NONCE_LEN);
    let cipher = XChaCha20Poly1305::new(enc_key.into());
    let nonce = XNonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| CloudCryptoError::UnwrapFailed)?;
    if plaintext.len() != KDF_OUTPUT_LEN {
        return Err(CloudCryptoError::WrongUnwrapLen(plaintext.len()));
    }
    let mut out = [0u8; KDF_OUTPUT_LEN];
    out.copy_from_slice(&plaintext);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chacha20poly1305::aead::OsRng;

    #[test]
    fn context_salt_has_no_separator() {
        let salt = [0x11u8; 16];
        let out = ctx_salt(&salt, "auth");
        assert_eq!(out.len(), 16 + 4);
        assert_eq!(&out[..16], &salt);
        assert_eq!(&out[16..], b"auth");
    }

    #[test]
    fn derive_auth_hash_is_deterministic() {
        let salt = [0u8; 16];
        let a = derive_auth_hash("correct horse battery staple", &salt).unwrap();
        let b = derive_auth_hash("correct horse battery staple", &salt).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn derive_auth_hash_differs_from_enc_key() {
        let salt = [0u8; 16];
        let auth = derive_auth_hash("password", &salt).unwrap();
        let enc = derive_enc_key("password", &salt).unwrap();
        assert_ne!(auth, enc, "context strings must separate the outputs");
    }

    #[test]
    fn derive_auth_hash_differs_from_recovery() {
        let salt = [0u8; 16];
        let auth = derive_auth_hash("password", &salt).unwrap();
        let recov = derive_recovery_key("password", &salt).unwrap();
        assert_ne!(auth, recov);
    }

    #[test]
    fn different_passwords_yield_different_auth_hashes() {
        let salt = [0u8; 16];
        let a = derive_auth_hash("pw1", &salt).unwrap();
        let b = derive_auth_hash("pw2", &salt).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn different_salts_yield_different_auth_hashes() {
        let a = derive_auth_hash("pw", &[1u8; 16]).unwrap();
        let b = derive_auth_hash("pw", &[2u8; 16]).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn xchacha_roundtrip_with_derived_key() {
        use chacha20poly1305::aead::rand_core::RngCore;
        let enc_key = derive_enc_key("pw", &[0u8; 16]).unwrap();
        let account_key = [42u8; 32];
        let mut nonce = [0u8; XCHACHA_NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        let cipher = XChaCha20Poly1305::new(&enc_key.into());
        let ct = cipher.encrypt(XNonce::from_slice(&nonce), &account_key[..]).unwrap();
        let mut wrapped = Vec::with_capacity(nonce.len() + ct.len());
        wrapped.extend_from_slice(&nonce);
        wrapped.extend_from_slice(&ct);
        let unwrapped = unwrap_account_key(&wrapped, &enc_key).unwrap();
        assert_eq!(unwrapped, account_key);
    }

    #[test]
    fn wrong_key_fails_unwrap() {
        use chacha20poly1305::aead::rand_core::RngCore;
        let right = derive_enc_key("right", &[0u8; 16]).unwrap();
        let wrong = derive_enc_key("wrong", &[0u8; 16]).unwrap();
        let account_key = [7u8; 32];
        let mut nonce = [0u8; XCHACHA_NONCE_LEN];
        OsRng.fill_bytes(&mut nonce);
        let cipher = XChaCha20Poly1305::new(&right.into());
        let ct = cipher.encrypt(XNonce::from_slice(&nonce), &account_key[..]).unwrap();
        let mut wrapped = nonce.to_vec();
        wrapped.extend_from_slice(&ct);
        assert!(matches!(
            unwrap_account_key(&wrapped, &wrong),
            Err(CloudCryptoError::UnwrapFailed)
        ));
    }

    #[test]
    fn truncated_wrap_rejected() {
        let key = [0u8; 32];
        let out = unwrap_account_key(&[0u8; 10], &key);
        assert!(matches!(out, Err(CloudCryptoError::WrappedTooShort(10))));
    }
}
