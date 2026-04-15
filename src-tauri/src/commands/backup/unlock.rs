//! Passphrase unlock + OS-keychain auto-unlock paths.

use super::config::{backup_status, build_raw_backend};
use super::BackupStatusPayload;
use crate::sync::app_backend;
use crate::sync::backend::SyncBackend;
use crate::sync::crypto::KeyMaterial;
use crate::sync::keychain;
use crate::sync::session::{SessionState, SyncSession};
use crate::sync::state::BackendKind;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn backup_unlock(
    app: AppHandle,
    session: State<'_, SessionState>,
    passphrase: String,
) -> Result<BackupStatusPayload, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let mut cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;

    // For hosted, prefer the account-canonical salt cached by
    // `cloud_signin` over whatever's persisted locally. This self-heals
    // any installation whose `sync-backend.json` was written before the
    // configure-time salt fix landed (see
    // `project_hosted_salt_canonical`). Fall through to the local cache
    // only if the keychain entry is missing (e.g. DBus down, fresh
    // restore from a backup file).
    if matches!(cfg.backend_kind, BackendKind::Hosted) {
        if let Ok(Some(canonical_b64)) = keychain::retrieve_cloud(keychain::CLOUD_KEY_KDF_SALT) {
            if canonical_b64 != cfg.salt_b64 {
                log::warn!(
                    "hosted backup salt drift: local cache differs from cloud-canonical salt — \
                     replacing local cache and re-deriving key"
                );
                cfg.salt_b64 = canonical_b64;
                if let Err(e) = app_backend::save(&app_data_dir, &cfg).await {
                    log::warn!("could not persist refreshed hosted salt: {e}");
                }
            }
        }
    }

    let salt = B64
        .decode(&cfg.salt_b64)
        .map_err(|e| format!("corrupt salt in app config: {e}"))?;
    let key = KeyMaterial::derive(&passphrase, &salt).map_err(|e| e.to_string())?;

    // Filesystem + S3 have a shared bucket root with a sync-meta.json
    // and cross-tome list that we can probe to validate the passphrase
    // up-front. Hosted doesn't — each tome is URL-scoped and there's
    // no `build_raw_backend` equivalent for hosted. Skip the probe;
    // if the passphrase is wrong, the first real sync will surface a
    // decrypt error.
    if !matches!(cfg.backend_kind, BackendKind::Hosted) {
        let raw = build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
        let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();
        let probes = raw_arc
            .list_prefix("tomes")
            .await
            .map_err(|e| e.to_string())?;
        if let Some(probe) = probes
            .iter()
            .find(|o| o.key.contains(".snap.enc") || o.key.contains(".op.enc"))
        {
            let (ciphertext, _etag) = raw_arc
                .get_object(&probe.key)
                .await
                .map_err(|e| e.to_string())?;
            crate::sync::crypto::decrypt(&key, &ciphertext).map_err(|_| {
                "wrong passphrase — could not decrypt existing backup data".to_string()
            })?;
        }
    }

    session
        .set(SyncSession {
            key: Arc::new(key),
        })
        .await;
    session.nudge();
    if let Err(e) = keychain::store(&passphrase) {
        log::warn!("could not refresh backup keychain entry: {e}");
    }
    backup_status(app, session).await
}

/// Try the OS keychain. Returns true if a stored passphrase unlocked the
/// backup. Silent on every failure so the UI can fall back to manual
/// unlock without spurious errors.
#[tauri::command]
pub async fn backup_try_auto_unlock(
    app: AppHandle,
    session: State<'_, SessionState>,
) -> Result<bool, String> {
    if session.current().await.is_some() {
        return Ok(true);
    }
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    if app_backend::load(&app_data_dir).await?.is_none() {
        return Ok(false);
    }
    let passphrase = match keychain::retrieve() {
        Ok(Some(p)) => p,
        _ => return Ok(false),
    };
    Ok(backup_unlock(app, session, passphrase).await.is_ok())
}
