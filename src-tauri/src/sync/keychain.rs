//! OS keychain integration for sync passphrases.
//!
//! Uses the `keyring` crate which routes to:
//! - **macOS**: Keychain Services
//! - **Windows**: Credential Manager
//! - **Linux**: Secret Service (gnome-keyring / kwallet) or kernel keyring
//!
//! All operations are best-effort — if the platform keychain is unavailable
//! (e.g. WSL without gnome-keyring), the helpers return errors that callers
//! are expected to swallow gracefully and fall back to manual entry.
//!
//! Each Tome stores its passphrase under a service prefix + the tome_id as
//! username, so multiple synced Tomes coexist without collision.

const SERVICE: &str = "vaelorium-sync";

#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("keychain unavailable: {0}")]
    Unavailable(String),

    #[error("keychain entry not found")]
    NotFound,
}

fn entry(tome_id: &str) -> Result<keyring::Entry, KeychainError> {
    keyring::Entry::new(SERVICE, tome_id).map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Persist a passphrase for `tome_id`. Overwrites any existing entry.
pub fn store(tome_id: &str, passphrase: &str) -> Result<(), KeychainError> {
    entry(tome_id)?
        .set_password(passphrase)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Retrieve a stored passphrase for `tome_id`, or `Ok(None)` if no entry exists.
pub fn retrieve(tome_id: &str) -> Result<Option<String>, KeychainError> {
    match entry(tome_id)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeychainError::Unavailable(e.to_string())),
    }
}

/// Remove the stored passphrase for `tome_id`. NotFound is treated as success.
pub fn delete(tome_id: &str) -> Result<(), KeychainError> {
    match entry(tome_id)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(KeychainError::Unavailable(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    // Skipping unit tests — the keyring backend is platform-specific and
    // tests would either require a real keychain (flaky in CI) or a mock
    // (defeats the integration purpose). The functions are thin wrappers
    // over a well-tested crate; bug surface area is in our usage, not here.
}
