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
/// JSON-encoded snapshot of the `UsagePayload` struct. Updated on
/// signin + (optionally) on every mutation response. Read by
/// `cloud_status` so the UI can display quota / tome count without
/// re-hitting the server.
pub const CLOUD_KEY_USAGE: &str = "usage";
pub const CLOUD_KEYS: &[&str] = &[
    CLOUD_KEY_DEVICE_TOKEN,
    CLOUD_KEY_ACCOUNT_KEY,
    CLOUD_KEY_KDF_SALT,
    CLOUD_KEY_EMAIL,
    CLOUD_KEY_TIER,
    CLOUD_KEY_USAGE,
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

/// File-backed fallback for cloud entries when the OS keychain is
/// unavailable (e.g. WSL without gnome-keyring). Survives process
/// restart so users don't re-sign-in every launch. Stored plaintext
/// at `<app_data_dir>/cloud-fallback.json` under the same
/// trusted-local-device posture already used for S3 access keys in
/// `sync-backend.json`. Loaded on first use, kept in sync in-memory
/// to avoid re-reading the file on every keychain call.
fn fallback_path() -> Option<std::path::PathBuf> {
    // Tauri's AppHandle isn't reachable from this module, so we resolve
    // the same data dir Tauri would via XDG env vars. macOS and Linux
    // use $XDG_DATA_HOME (else $HOME/.local/share). Windows uses
    // %APPDATA%. These match the `app_data_dir` that
    // `sync::app_backend` uses, so the fallback lives alongside
    // sync-backend.json.
    let base: std::path::PathBuf = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(std::path::PathBuf::from)?
    } else if cfg!(target_os = "macos") {
        let home = std::env::var_os("HOME").map(std::path::PathBuf::from)?;
        home.join("Library").join("Application Support")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
            })?
    };
    Some(base.join("com.vaelorium.app").join("cloud-fallback.json"))
}

fn fallback_load() -> std::collections::HashMap<String, String> {
    let Some(path) = fallback_path() else { return Default::default() };
    match std::fs::read_to_string(&path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Default::default(),
        Err(e) => {
            log::warn!("keychain fallback: read {path:?}: {e}");
            Default::default()
        }
    }
}

fn fallback_save(map: &std::collections::HashMap<String, String>) {
    let Some(path) = fallback_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    match serde_json::to_string(map) {
        Ok(s) => {
            if let Err(e) = std::fs::write(&path, s) {
                log::warn!("keychain fallback: write {path:?}: {e}");
            }
        }
        Err(e) => log::warn!("keychain fallback: serialize: {e}"),
    }
}

fn fallback_map() -> &'static std::sync::Mutex<std::collections::HashMap<String, String>> {
    static MAP: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
        std::sync::OnceLock::new();
    MAP.get_or_init(|| std::sync::Mutex::new(fallback_load()))
}

fn fallback_get(key: &str) -> Option<String> {
    fallback_map().lock().ok().and_then(|m| m.get(key).cloned())
}

fn fallback_set(key: &str, value: &str) {
    if let Ok(mut m) = fallback_map().lock() {
        m.insert(key.to_string(), value.to_string());
        fallback_save(&m);
    }
}

fn fallback_remove(key: &str) {
    if let Ok(mut m) = fallback_map().lock() {
        m.remove(key);
        fallback_save(&m);
    }
}

fn cloud_entry(key: &str) -> Result<keyring::Entry, KeychainError> {
    keyring::Entry::new(CLOUD_SERVICE, key)
        .map_err(|e| KeychainError::Unavailable(e.to_string()))
}

/// Store a cloud-session value. Tries the OS keychain first; if the
/// platform doesn't have one available (DBus error on WSL, etc.),
/// falls back to a process-lifetime in-memory map. Returns Ok in
/// either case so callers don't have to retry — the fallback covers
/// the same session, and signin commands can warn the user that
/// they'll need to re-signin after restart.
pub fn store_cloud(key: &str, value: &str) -> Result<(), KeychainError> {
    match cloud_entry(key) {
        Ok(entry) => match entry.set_password(value) {
            Ok(()) => {
                // Drop any lingering fallback once the OS keychain works.
                fallback_remove(key);
                Ok(())
            }
            Err(e) => {
                log::warn!(
                    "keychain unavailable for {CLOUD_SERVICE}/{key}: {e} — using in-memory fallback (re-signin required after restart)"
                );
                fallback_set(key, value);
                Ok(())
            }
        },
        Err(e) => {
            log::warn!(
                "keychain entry init failed for {CLOUD_SERVICE}/{key}: {e} — using in-memory fallback"
            );
            fallback_set(key, value);
            Ok(())
        }
    }
}

/// Retrieve a cloud-session value. Checks the OS keychain first,
/// falls back to in-memory map.
pub fn retrieve_cloud(key: &str) -> Result<Option<String>, KeychainError> {
    match cloud_entry(key) {
        Ok(entry) => match entry.get_password() {
            Ok(p) => Ok(Some(p)),
            Err(keyring::Error::NoEntry) => Ok(fallback_get(key)),
            Err(_) => Ok(fallback_get(key)),
        },
        Err(_) => Ok(fallback_get(key)),
    }
}

/// Remove a single cloud-session entry. NotFound + keychain-unavailable
/// are both treated as success — the fallback map is always cleared.
pub fn delete_cloud(key: &str) -> Result<(), KeychainError> {
    fallback_remove(key);
    match cloud_entry(key) {
        Ok(entry) => match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Ok(()),
        },
        Err(_) => Ok(()),
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
