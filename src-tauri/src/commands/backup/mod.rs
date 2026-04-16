//! App-global backup destination commands.
//!
//! The "backup" is the shared backend (S3 bucket / filesystem folder) that
//! every Tome with sync enabled writes into, namespaced by `tomes/{id}/`.
//! Configured once per app installation; one passphrase across all Tomes.
//!
//! Split into submodules by concern:
//! - [`config`]: connect / disconnect / status / rename device / backend factory
//! - [`unlock`]: passphrase unlock + OS-keychain auto-unlock
//! - [`restore`]: enumerate restorable Tomes + download a snapshot into place
//! - [`delete`]: remove a Tome's entire presence from the backup

use serde::{Deserialize, Serialize};

pub mod config;
pub mod delete;
pub mod restore;
pub mod unlock;

// `#[tauri::command]` fns are registered in `invoke_handler![]` via
// their explicit submodule paths (`commands::backup::config::...`
// etc.) — Tauri's macro expansion keys on the module the fn lives in,
// so re-exporting `pub use` from here wouldn't work for registration.
// The `build_raw_backend` helper is still shared across submodules;
// it's `pub` in `config` and consumers import it directly.

// --- Shared payload types ---------------------------------------------------

#[derive(Debug, Serialize)]
pub struct BackupStatusPayload {
    /// `sync-backend.json` exists in the app data dir.
    pub configured: bool,
    /// `configured` is true but no in-memory key cached. User must unlock.
    pub locked: bool,
    pub backend_kind: Option<String>,
    /// Human-readable summary (e.g. filesystem path, S3 bucket name).
    pub backend_summary: Option<String>,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ConfigureInput {
    pub backend_kind: String,
    pub backend_config: serde_json::Value,
    pub passphrase: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DeviceNameInput {
    pub device_name: String,
}

#[derive(Debug, Serialize)]
pub struct RestorableTome {
    pub tome_uuid: String,
    pub snapshot_id: String,
    pub name: String,
    pub description: Option<String>,
    pub size_bytes: u64,
    pub last_modified: String,
}

#[derive(Debug, Serialize)]
pub struct RestoredTome {
    pub path: String,
    pub name: String,
    pub tome_uuid: String,
    /// Non-fatal warning from restore (e.g. journal replay failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreInput {
    pub tome_uuid: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteTomeInput {
    pub tome_uuid: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteTomeResult {
    pub deleted_objects: u64,
    pub deleted_bytes: u64,
}

// --- Shared helpers ---------------------------------------------------------

/// Append a 4-char hex stub derived from `device_id` to `name` when the
/// name doesn't already end with a parenthesized 4-hex disambiguator.
/// Prevents two devices named "My Laptop" from being indistinguishable
/// in conflict logs or activity views while keeping display names short.
/// Used both by local `backup_configure` (for sync_op attribution) and
/// `cloud_signin` (for the server-side device registry).
pub fn ensure_device_name_stub(name: &str, device_id: uuid::Uuid) -> String {
    let trimmed = name.trim();
    // Already has a "(xxxx)" stub at the tail? Leave alone so renames
    // don't accumulate stubs like "Laptop (a3f2) (b7c1)".
    let has_stub = trimmed
        .rsplit_once('(')
        .and_then(|(_, rest)| rest.strip_suffix(')'))
        .map(|inner| inner.len() == 4 && inner.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or(false);
    if has_stub {
        return trimmed.to_string();
    }
    let hex = device_id.simple().to_string();
    let stub: String = hex.chars().take(4).collect();
    format!("{trimmed} ({stub})")
}

#[cfg(test)]
mod stub_tests {
    use super::ensure_device_name_stub;
    use uuid::Uuid;

    fn uuid_from_hex(hex: &str) -> Uuid {
        Uuid::parse_str(hex).unwrap()
    }

    #[test]
    fn appends_stub_when_missing() {
        let id = uuid_from_hex("a3f2b7c1-0000-0000-0000-000000000000");
        let out = ensure_device_name_stub("My Laptop", id);
        assert_eq!(out, "My Laptop (a3f2)");
    }

    #[test]
    fn preserves_existing_stub() {
        let id = uuid_from_hex("deadbeef-0000-0000-0000-000000000000");
        let out = ensure_device_name_stub("Laptop (a3f2)", id);
        assert_eq!(out, "Laptop (a3f2)");
    }

    #[test]
    fn rejects_non_hex_suffix_as_stub() {
        let id = uuid_from_hex("a3f2b7c1-0000-0000-0000-000000000000");
        let out = ensure_device_name_stub("Thing (home)", id);
        assert_eq!(out, "Thing (home) (a3f2)");
    }

    #[test]
    fn trims_whitespace() {
        let id = uuid_from_hex("a3f2b7c1-0000-0000-0000-000000000000");
        let out = ensure_device_name_stub("  Laptop  ", id);
        assert_eq!(out, "Laptop (a3f2)");
    }
}
