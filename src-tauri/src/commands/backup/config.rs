//! Connect / disconnect / status / rename device + the raw-backend factory.

use super::{ensure_device_name_stub, BackupStatusPayload, ConfigureInput, DeviceNameInput};
use crate::sync::app_backend::{self, AppBackendConfig};
use crate::sync::backend::s3::{S3Backend, S3Config};
use crate::sync::backend::{filesystem::FilesystemBackend, SyncBackend};
use crate::sync::crypto::{generate_salt, KeyMaterial};
use crate::sync::keychain;
use crate::sync::remote_meta::{self, RemoteMeta};
use crate::sync::session::{SessionState, SyncSession};
use crate::sync::state::BackendKind;
use base64::{engine::general_purpose::STANDARD as B64, Engine as _};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn backup_configure(
    app: AppHandle,
    session: State<'_, SessionState>,
    input: ConfigureInput,
) -> Result<BackupStatusPayload, String> {
    let backend_kind = BackendKind::from_str(&input.backend_kind)
        .ok_or_else(|| format!("unknown backend_kind: {}", input.backend_kind))?;

    // Hosted (Vaelorium Cloud) takes a different validation path — it
    // has no shared bucket root to probe. Auth lives in the keychain
    // (set by cloud_signin); the blob passphrase still derives a
    // KeyMaterial exactly as for Filesystem/S3, but we generate a fresh
    // salt (or reuse the existing app-global one) without a backend
    // round-trip.
    let (salt, key, remote_for_meta) = if matches!(backend_kind, BackendKind::Hosted) {
        // Require a signed-in cloud session.
        let _token = crate::commands::cloud::require_device_token()?;
        let existing = app_backend::load(
            &app.path().app_data_dir().map_err(|e| format!("app data dir: {e}"))?,
        )
        .await
        .ok()
        .flatten();
        let salt: Vec<u8> = match existing.as_ref() {
            Some(e) => B64
                .decode(&e.salt_b64)
                .map_err(|e| format!("existing salt decode: {e}"))?,
            None => generate_salt().to_vec(),
        };
        let key = KeyMaterial::derive(&input.passphrase, &salt).map_err(|e| e.to_string())?;
        (salt, key, None)
    } else {
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
        (salt, key, Some(remote))
    };
    let _ = remote_for_meta; // suppress unused; only useful above

    // Persist the app-global config. Reuse the existing device_id if the
    // user is reconnecting to the same bucket — otherwise ops from this
    // device would start showing up under a new attribution.
    let existing = app_backend::load(
        &app.path().app_data_dir().map_err(|e| format!("app data dir: {e}"))?,
    )
    .await
    .ok()
    .flatten();
    let raw_name = input.device_name.unwrap_or_else(|| {
        existing
            .as_ref()
            .map(|c| c.device_name.clone())
            .unwrap_or_else(|| std::env::var("HOSTNAME").unwrap_or_else(|_| "Vaelorium Device".into()))
    });
    let device_id = existing.as_ref().map(|c| c.device_id).unwrap_or_else(uuid::Uuid::new_v4);
    let device_name = ensure_device_name_stub(&raw_name, device_id);
    // Hosted: decorate backend_config with email + tier from keychain
    // so backup_status can render "Cloud <tier> — <email>" without
    // touching the keychain on every status call.
    let backend_config = if matches!(backend_kind, BackendKind::Hosted) {
        let email = keychain::retrieve_cloud(keychain::CLOUD_KEY_EMAIL)
            .ok()
            .flatten()
            .unwrap_or_default();
        let tier = keychain::retrieve_cloud(keychain::CLOUD_KEY_TIER)
            .ok()
            .flatten();
        let mut obj = serde_json::Map::new();
        if !email.is_empty() {
            obj.insert("email".to_string(), serde_json::Value::String(email));
        }
        if let Some(t) = tier {
            if !t.is_empty() {
                obj.insert("tier".to_string(), serde_json::Value::String(t));
            }
        }
        serde_json::Value::Object(obj)
    } else {
        input.backend_config.clone()
    };

    // Preserve any existing device_token on reconnect / reconfigure;
    // only cloud_signin / cloud_signout mutate it.
    let device_token = existing.as_ref().and_then(|c| c.device_token.clone());
    let cfg = AppBackendConfig {
        backend_kind,
        backend_config,
        salt_b64: B64.encode(&salt),
        device_id,
        device_name,
        created_at: existing.as_ref().map(|c| c.created_at).unwrap_or_else(chrono::Utc::now),
        device_token,
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
                BackendKind::Hosted => {
                    let email = c
                        .backend_config
                        .get("email")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Vaelorium Cloud");
                    let tier = c
                        .backend_config
                        .get("tier")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    if tier.is_empty() {
                        format!("Cloud — {email}")
                    } else {
                        format!("Cloud {tier} — {email}")
                    }
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
    cfg.device_name = ensure_device_name_stub(&input.device_name, cfg.device_id);
    app_backend::save(&app_data_dir, &cfg).await?;
    backup_status(app, session).await
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
        BackendKind::Hosted => {
            // Hosted doesn't have a shared bucket root — each Tome is
            // addressed per-URL (/v1/tomes/<uuid>/...) and auth is a
            // bearer token held in the OS keychain, not a signing key.
            // Raw (pre-prefix) operations aren't meaningful; engine code
            // should use `sync::commands::build_tome_backend` which
            // constructs a HostedBackend with the tome_uuid baked in.
            Err("hosted backend requires a tome_uuid; use build_tome_backend".to_string())
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
