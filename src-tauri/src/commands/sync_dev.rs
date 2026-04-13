//! M7 Sync — DEV-ONLY commands for end-to-end manual testing.
//!
//! These commands are NOT user-facing. They exist so the engine can be driven
//! from the Tauri dev console while Phase 3 (Settings UI + ConflictResolver)
//! is still being built. Phase 3 will replace the dev panel with the real
//! Settings → Sync UX and these commands should disappear before any release.

use crate::db::{self, ManagedDb};
use crate::sync::backend::filesystem::FilesystemBackend;
use crate::sync::crypto::{generate_salt, KeyMaterial};
use crate::sync::state::{BackendKind, SyncConfig};
use crate::sync::{sync_tome_once, SyncOutcome};
use serde_json::json;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

/// Configure the active Tome for filesystem-backed sync, then run one sync
/// cycle. Idempotent: re-running with the same passphrase derives the same key.
///
/// **Dev only** — there is no UI to validate inputs, no error recovery,
/// no protection against losing the passphrase.
#[tauri::command]
pub async fn sync_dev_filesystem(
    managed: State<'_, ManagedDb>,
    tome_id: String,
    backend_path: String,
    passphrase: String,
) -> Result<SyncOutcome, String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Find or create sync_config for this Tome.
    let existing = SyncConfig::load(&pool, &tome_id)
        .await
        .map_err(|e| e.to_string())?;
    let (salt, device_id) = match existing {
        Some(cfg) => (cfg.passphrase_salt, cfg.device_id),
        None => {
            let salt = generate_salt().to_vec();
            let device_id = Uuid::new_v4();
            let cfg = SyncConfig {
                tome_id: tome_id.clone(),
                enabled: true,
                backend_type: BackendKind::Filesystem,
                backend_config: json!({ "path": backend_path }),
                passphrase_salt: salt.clone(),
                device_id,
                device_name: hostname_or("Vaelorium Device"),
                schema_version: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            };
            cfg.save(&pool).await.map_err(|e| e.to_string())?;
            (salt, device_id)
        }
    };
    let _ = device_id; // unused in this dev command, but kept for clarity

    let key = KeyMaterial::derive(&passphrase, &salt).map_err(|e| e.to_string())?;
    let backend = FilesystemBackend::new(PathBuf::from(backend_path))
        .await
        .map_err(|e| e.to_string())?;

    sync_tome_once(&pool, &tome_id, &key, &backend)
        .await
        .map_err(|e| e.to_string())
}

fn hostname_or(default: &str) -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| default.to_string())
}
