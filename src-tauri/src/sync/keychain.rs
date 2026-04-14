//! OS keychain integration.
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
//! Two separate services are used to keep concerns isolated:
//! - `vaelorium-sync` / `backup` — the app-global backup passphrase (M7).
//! - `vaelorium-cloud` / `{device-token, account-key, kdf-salt, email}` —
//!   Vaelorium Cloud session state (M5). Multiple named entries under a
//!   single service, cleared on signout.

const SERVICE: &str = "vaelorium-sync";
const BACKUP_KEY: &str = "backup";

const CLOUD_SERVICE: &str = "vaelorium-cloud";
pub const CLOUD_KEY_DEVICE_TOKEN: &str = "device-token";
pub const CLOUD_KEY_ACCOUNT_KEY: &str = "account-key";
pub const CLOUD_KEY_KDF_SALT: &str = "kdf-salt";
pub const CLOUD_KEY_EMAIL: &str = "email";
pub const CLOUD_KEY_TIER: &str = "tier";
pub const CLOUD_KEYS: &[&str] = &[
    CLOUD_KEY_DEVICE_TOKEN,
    CLOUD_KEY_ACCOUNT_KEY,
    CLOUD_KEY_KDF_SALT,
    CLOUD_KEY_EMAIL,
    CLOUD_KEY_TIER,
];

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

// ----- Vaelorium Cloud (M5) -----

fn cloud_entry(key: &str) -> Result<keyring::Entry, KeychainError> {
    keyring::Entry::new(CLOUD_SERVICE, key)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Store a cloud-session value. Overwrites any existing entry.
pub fn store_cloud(key: &str, value: &str) -> Result<(), KeychainError> {
    cloud_entry(key)?
        .set_password(value)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Retrieve a cloud-session value, or `Ok(None)` if no entry exists.
pub fn retrieve_cloud(key: &str) -> Result<Option<String>, KeychainError> {
    match cloud_entry(key)?.get_password() {
        Ok(p) => Ok(Some(p)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(KeychainError::Unavailable(e.to_string())),
    }
}

/// Remove a single cloud-session entry. NotFound is treated as success.
pub fn delete_cloud(key: &str) -> Result<(), KeychainError> {
    match cloud_entry(key)?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(KeychainError::Unavailable(e.to_string())),
    }
}

/// Best-effort clear of all cloud-session entries. Called on signout —
/// individual failures are logged but don't abort the sweep.
pub fn clear_all_cloud() {
    for key in CLOUD_KEYS {
        if let Err(e) = delete_cloud(key) {
            log::warn!("keychain: clear_all_cloud({key}): {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    // Skipping unit tests — the keyring backend is platform-specific and
    // tests would either require a real keychain (flaky in CI) or a mock
    // (defeats the integration purpose). The functions are thin wrappers
    // over a well-tested crate; bug surface area is in our usage, not here.
}
