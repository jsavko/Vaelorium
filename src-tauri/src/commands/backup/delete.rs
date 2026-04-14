//! Remove a Tome's entire presence from the configured backup.

use super::config::build_raw_backend;
use super::{DeleteTomeInput, DeleteTomeResult};
use crate::sync::app_backend;
use crate::sync::backend::SyncBackend;
use crate::sync::session::SessionState;
use crate::sync::state::BackendKind;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// Remove a Tome's entire presence from the configured backup.
/// Backend-agnostic: Hosted calls `DELETE /v1/tomes/<uuid>` in one
/// shot; Filesystem + S3 iterate `list_prefix("tomes/<uuid>")` +
/// `delete_object` for every key. Safe to call on a Tome that's
/// currently sync-enabled locally, but the caller should disable
/// sync first — otherwise the next sync tick will re-upload everything
/// that was just deleted.
#[tauri::command]
pub async fn backup_delete_tome(
    app: AppHandle,
    session: State<'_, SessionState>,
    input: DeleteTomeInput,
) -> Result<DeleteTomeResult, String> {
    // Require an unlocked session for all backends (hosted uses the
    // device token, filesystem/S3 need the key to not just delete
    // blobs but also log the intent coherently).
    if session.current().await.is_none() {
        return Err("backup is locked".to_string());
    }
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;

    match cfg.backend_kind {
        BackendKind::Hosted => {
            let token = crate::commands::cloud::require_device_token_with_app(&app)?;
            let client =
                crate::sync::backend::hosted::HostedClient::new().map_err(|e| e.to_string())?;
            let (deleted_objects, deleted_bytes, _usage) = client
                .delete_tome(&token, &input.tome_uuid)
                .await
                .map_err(|e| e.to_string())?;
            Ok(DeleteTomeResult {
                deleted_objects,
                deleted_bytes,
            })
        }
        BackendKind::Filesystem | BackendKind::S3 => {
            // Build the raw backend + enumerate + delete loop. Not the
            // fastest for S3 (N round-trips) but rare (user decommissioning
            // a Tome), and avoids per-backend special casing.
            let raw = build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
            let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();
            let prefix = format!("tomes/{}", input.tome_uuid);
            let infos = raw_arc
                .list_prefix(&prefix)
                .await
                .map_err(|e| e.to_string())?;
            let mut deleted_objects = 0u64;
            let mut deleted_bytes = 0u64;
            for info in &infos {
                match raw_arc.delete_object(&info.key).await {
                    Ok(()) => {
                        deleted_objects += 1;
                        deleted_bytes += info.size;
                    }
                    Err(e) => log::warn!(
                        "backup_delete_tome: failed to delete {}: {}",
                        info.key,
                        e
                    ),
                }
            }
            Ok(DeleteTomeResult {
                deleted_objects,
                deleted_bytes,
            })
        }
    }
}
