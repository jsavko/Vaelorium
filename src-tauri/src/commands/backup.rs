//! App-global backup destination commands.
//!
//! The "backup" is the shared backend (S3 bucket / filesystem folder) that
//! every Tome with sync enabled writes into, namespaced by `tomes/{id}/`.
//! Configured once per app installation; one passphrase across all Tomes.

use crate::sync::app_backend::{self, AppBackendConfig};
use crate::sync::backend::s3::{S3Backend, S3Config};
use crate::sync::backend::{filesystem::FilesystemBackend, SyncBackend};
use crate::sync::crypto::{generate_salt, KeyMaterial};
use crate::sync::keychain;
use crate::sync::remote_meta::{self, RemoteMeta};
use crate::sync::session::{SessionState, SyncSession};
use crate::sync::state::BackendKind;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

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

#[tauri::command]
pub async fn backup_configure(
    app: AppHandle,
    session: State<'_, SessionState>,
    input: ConfigureInput,
) -> Result<BackupStatusPayload, String> {
    let backend_kind = BackendKind::from_str(&input.backend_kind)
        .ok_or_else(|| format!("unknown backend_kind: {}", input.backend_kind))?;

    // Build the raw backend (no prefix; sync-meta.json is at root).
    let raw = build_raw_backend(backend_kind, &input.backend_config).await?;
    let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();

    // Look for existing sync-meta.json (returning device or fresh bucket).
    let remote = remote_meta::fetch(raw_arc.as_ref()).await?;
    let salt: Vec<u8> = match &remote {
        Some(meta) => meta.salt()?,
        None => generate_salt().to_vec(),
    };

    let key = KeyMaterial::derive(&input.passphrase, &salt).map_err(|e| e.to_string())?;

    // Validate passphrase against any existing Tome's data on the bucket.
    // We probe per-Tome subtrees because the new layout puts everything
    // under tomes/*/{snapshots,journal}/.
    let probe_objects = raw_arc
        .list_prefix("tomes")
        .await
        .map_err(|e| e.to_string())?;
    if let Some(probe) = probe_objects
        .iter()
        .find(|o| o.key.contains(".snap.enc") || o.key.contains(".op.enc"))
    {
        let (ciphertext, _etag) = raw_arc
            .get_object(&probe.key)
            .await
            .map_err(|e| e.to_string())?;
        crate::sync::crypto::decrypt(&key, &ciphertext).map_err(|_| {
            "passphrase does not match the existing backup data — \
             the passphrase is wrong."
                .to_string()
        })?;
    }

    // First-ever connection to this bucket → publish meta.
    if remote.is_none() {
        remote_meta::put(raw_arc.as_ref(), &RemoteMeta::new(&salt)).await?;
    }

    // Persist the app-global config. Reuse the existing device_id if the
    // user is reconnecting to the same bucket — otherwise ops from this
    // device would start showing up under a new attribution.
    let existing = app_backend::load(
        &app.path().app_data_dir().map_err(|e| format!("app data dir: {e}"))?,
    )
    .await
    .ok()
    .flatten();
    let device_name = input.device_name.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|c| c.device_name.clone())
            .unwrap_or_else(|| std::env::var("HOSTNAME").unwrap_or_else(|_| "Vaelorium Device".into()))
    });
    let cfg = AppBackendConfig {
        backend_kind,
        backend_config: input.backend_config.clone(),
        salt_b64: B64.encode(&salt),
        device_id: existing.as_ref().map(|c| c.device_id).unwrap_or_else(uuid::Uuid::new_v4),
        device_name,
        created_at: existing.as_ref().map(|c| c.created_at).unwrap_or_else(chrono::Utc::now),
    };
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    app_backend::save(&app_data_dir, &cfg).await?;

    // Cache key + persist passphrase to OS keychain (best-effort).
    session
        .set(SyncSession {
            key: Arc::new(key),
        })
        .await;
    session.nudge();
    if let Err(e) = keychain::store(&input.passphrase) {
        log::warn!("could not store backup passphrase in keychain: {e}");
    }

    backup_status(app, session).await
}

#[tauri::command]
pub async fn backup_disconnect(
    app: AppHandle,
    session: State<'_, SessionState>,
) -> Result<BackupStatusPayload, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    app_backend::clear(&app_data_dir).await?;
    session.clear().await;
    if let Err(e) = keychain::delete() {
        log::warn!("could not delete backup keychain entry: {e}");
    }
    backup_status(app, session).await
}

#[tauri::command]
pub async fn backup_status(
    app: AppHandle,
    session: State<'_, SessionState>,
) -> Result<BackupStatusPayload, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let cfg = app_backend::load(&app_data_dir).await?;
    let configured = cfg.is_some();
    let locked = configured && session.current().await.is_none();
    let (backend_kind, backend_summary, device_name) = match &cfg {
        Some(c) => {
            let summary = match c.backend_kind {
                BackendKind::Filesystem => c
                    .backend_config
                    .get("path")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_default(),
                BackendKind::S3 => {
                    let bucket = c
                        .backend_config
                        .get("bucket")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let endpoint = c
                        .backend_config
                        .get("endpoint")
                        .and_then(|v| v.as_str())
                        .unwrap_or("AWS");
                    format!("{} on {}", bucket, endpoint)
                }
            };
            (
                Some(c.backend_kind.as_str().to_string()),
                Some(summary),
                Some(c.device_name.clone()),
            )
        }
        None => (None, None, None),
    };
    Ok(BackupStatusPayload {
        configured,
        locked,
        backend_kind,
        backend_summary,
        device_name,
    })
}

/// Rename this device. Persists to sync-backend.json. Does NOT rewrite
/// already-uploaded ops — they keep the old device_id (which may have
/// been associated with the prior name). Future ops get the new name.
#[tauri::command]
pub async fn backup_set_device_name(
    app: AppHandle,
    session: State<'_, SessionState>,
    input: DeviceNameInput,
) -> Result<BackupStatusPayload, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let mut cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;
    cfg.device_name = input.device_name;
    app_backend::save(&app_data_dir, &cfg).await?;
    backup_status(app, session).await
}

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
    let cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;

    let salt = B64
        .decode(&cfg.salt_b64)
        .map_err(|e| format!("corrupt salt in app config: {e}"))?;
    let key = KeyMaterial::derive(&passphrase, &salt).map_err(|e| e.to_string())?;

    // Validate via probe-decrypt against any existing data.
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
        crate::sync::crypto::decrypt(&key, &ciphertext)
            .map_err(|_| "wrong passphrase — could not decrypt existing backup data".to_string())?;
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

/// Construct an unwrapped backend for raw bucket-root operations
/// (sync-meta.json, multi-Tome listing). Engine code uses
/// `crate::commands::sync::build_tome_backend` instead, which wraps with
/// the per-Tome prefix.
pub async fn build_raw_backend(
    kind: BackendKind,
    config: &serde_json::Value,
) -> Result<Box<dyn SyncBackend + Send + Sync>, String> {
    match kind {
        BackendKind::Filesystem => {
            let path = config
                .get("path")
                .and_then(|v| v.as_str())
                .ok_or_else(|| "filesystem backend requires `path`".to_string())?;
            Ok(Box::new(
                FilesystemBackend::new(PathBuf::from(path))
                    .await
                    .map_err(|e| e.to_string())?,
            ))
        }
        BackendKind::S3 => {
            let s3_cfg = parse_s3_config(config)?;
            Ok(Box::new(
                S3Backend::new(s3_cfg).await.map_err(|e| e.to_string())?,
            ))
        }
    }
}

fn parse_s3_config(config: &serde_json::Value) -> Result<S3Config, String> {
    fn s(v: &serde_json::Value, k: &str) -> Result<String, String> {
        v.get(k)
            .and_then(|x| x.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| format!("s3 backend requires `{k}`"))
    }
    fn s_opt(v: &serde_json::Value, k: &str) -> Option<String> {
        v.get(k)
            .and_then(|x| x.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
    }
    Ok(S3Config {
        endpoint: s_opt(config, "endpoint"),
        region: s(config, "region")?,
        bucket: s(config, "bucket")?,
        access_key: s(config, "access_key")?,
        secret_key: s(config, "secret_key")?,
        prefix: s_opt(config, "prefix"),
    })
}

// re-export of json helper for tests (kept to avoid unused-import warnings).
#[allow(dead_code)]
fn _unused_json_marker() -> serde_json::Value {
    json!({})
}
