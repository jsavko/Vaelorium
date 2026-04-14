//! App-global backup backend configuration.
//!
//! One backup destination per app installation, configured once in
//! Settings → Backup. All Tomes that opt into backup share these
//! credentials and namespace their data under `tomes/{tome_id}/` in the
//! shared backend.
//!
//! Persisted as `sync-backend.json` in the Tauri app data directory. The
//! passphrase is NEVER stored here — it lives in the OS keychain.

use crate::sync::state::BackendKind;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs;

const FILENAME: &str = "sync-backend.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBackendConfig {
    pub backend_kind: BackendKind,
    /// JSON-encoded backend-specific config (path / S3 creds).
    pub backend_config: serde_json::Value,
    /// Cached salt — authoritative copy lives in bucket-root sync-meta.json,
    /// but we keep a local copy to derive the key without a round-trip.
    pub salt_b64: String,
    /// App-global device identity. All Tomes on this device share the same
    /// (id, name) in their op attribution. Defaulted on load so configs
    /// written before these fields existed don't fail to deserialize.
    #[serde(default = "uuid::Uuid::new_v4")]
    pub device_id: uuid::Uuid,
    #[serde(default = "default_device_name")]
    pub device_name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Hosted-cloud device token (JWT-equivalent). Only populated when
    /// `backend_kind == Hosted` and the user is signed in. Persists
    /// alongside the rest of backup config so a sign-in survives
    /// process restarts on any platform, even those without a working
    /// OS keychain. Trusted-local-device posture (same as the
    /// S3 access keys already stored in this file).
    ///
    /// Mirrors the `device-token` keychain entry — whichever source
    /// has a value is authoritative. Cleared on `cloud_signout`.
    #[serde(default)]
    pub device_token: Option<String>,
}

fn default_device_name() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "Vaelorium Device".into())
}

fn config_path(app_data_dir: &std::path::Path) -> PathBuf {
    app_data_dir.join(FILENAME)
}

/// Load the app-global backend config from the app data dir.
/// `Ok(None)` if no backup has been configured yet.
pub async fn load(app_data_dir: &std::path::Path) -> Result<Option<AppBackendConfig>, String> {
    let path = config_path(app_data_dir);
    match fs::read(&path).await {
        Ok(bytes) => {
            let cfg: AppBackendConfig = serde_json::from_slice(&bytes)
                .map_err(|e| format!("corrupt sync-backend.json: {e}"))?;
            Ok(Some(cfg))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("reading sync-backend.json: {e}")),
    }
}

/// Save the app-global backend config to the app data dir.
pub async fn save(
    app_data_dir: &std::path::Path,
    cfg: &AppBackendConfig,
) -> Result<(), String> {
    let path = config_path(app_data_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("creating app data dir: {e}"))?;
    }
    let bytes = serde_json::to_vec_pretty(cfg)
        .map_err(|e| format!("serializing sync-backend.json: {e}"))?;
    fs::write(&path, bytes)
        .await
        .map_err(|e| format!("writing sync-backend.json: {e}"))?;
    Ok(())
}

/// Delete the app-global backend config (called on Disconnect).
pub async fn clear(app_data_dir: &std::path::Path) -> Result<(), String> {
    let path = config_path(app_data_dir);
    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(format!("removing sync-backend.json: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;

    fn sample() -> AppBackendConfig {
        AppBackendConfig {
            backend_kind: BackendKind::Filesystem,
            backend_config: serde_json::json!({"path": "/tmp/sync"}),
            salt_b64: "AAAAAAAAAAAAAAAAAAAAAA==".to_string(),
            device_id: uuid::Uuid::new_v4(),
            device_name: "Test Device".to_string(),
            created_at: Utc::now(),
            device_token: None,
        }
    }

    #[tokio::test]
    async fn load_returns_none_when_missing() {
        let dir = tempdir().unwrap();
        assert!(load(dir.path()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn save_then_load_roundtrip() {
        let dir = tempdir().unwrap();
        let cfg = sample();
        save(dir.path(), &cfg).await.unwrap();
        let loaded = load(dir.path()).await.unwrap().unwrap();
        assert_eq!(loaded.backend_kind, cfg.backend_kind);
        assert_eq!(loaded.backend_config, cfg.backend_config);
        assert_eq!(loaded.salt_b64, cfg.salt_b64);
    }

    #[tokio::test]
    async fn clear_removes_file_idempotent() {
        let dir = tempdir().unwrap();
        save(dir.path(), &sample()).await.unwrap();
        clear(dir.path()).await.unwrap();
        clear(dir.path()).await.unwrap(); // idempotent
        assert!(load(dir.path()).await.unwrap().is_none());
    }
}
