//! OS keychain integration for the app-global backup passphrase.
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
//! One entry per app: `(SERVICE, BACKUP_KEY)`. The passphrase is shared
//! across every Tome that opts into backup.

const SERVICE: &str = "vaelorium-sync";
const BACKUP_KEY: &str = "backup";

#[derive(Debug, thiserror::Error)]
pub enum KeychainError {
    #[error("keychain unavailable: {0}")]
    Unavailable(String),

    #[error("keychain entry not found")]
    NotFound,
}

fn entry() -> Result<keyring::Entry, KeychainError> {
    keyring::Entry::new(SERVICE, BACKUP_KEY)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Persist the app-global backup passphrase. Overwrites any existing entry.
pub fn store(passphrase: &str) -> Result<(), KeychainError> {
    entry()?
        .set_password(passphrase)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Retrieve the stored backup passphrase, or `Ok(None)` if no entry exists.
pub fn retrieve() -> Result<Option<String>, KeychainError> {
    match entry()?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeychainError::Unavailable(e.to_string())),
    }
}

/// Remove the stored backup passphrase. NotFound is treated as success.
pub fn delete() -> Result<(), KeychainError> {
    match entry()?.delete_credential() {
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
