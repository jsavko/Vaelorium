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

// re-export of json helper for tests (kept to avoid unused-import warnings).
#[allow(dead_code)]
fn _unused_json_marker() -> serde_json::Value {
    json!({})
}

/// Append a 4-char hex stub derived from `device_id` to `name` when the
/// name doesn't already end with a parenthesized 4-hex disambiguator.
/// Prevents two devices named "My Laptop" from being indistinguishable
/// in conflict logs or activity views while keeping display names short.
fn ensure_device_name_stub(name: &str, device_id: uuid::Uuid) -> String {
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
}

/// List Tomes available for restore on this backend. Discovers via
/// `tomes/<uuid>/snapshots/*.snap.enc` keys, then downloads + decrypts the
/// latest snapshot for each to extract the display name from
/// `tome_metadata`.
///
/// Eager name extraction is acceptable for MVP — typical user has 1-3
/// Tomes, snapshots are gzipped + tiny (~10 KB empty, scaling with content).
#[tauri::command]
pub async fn backup_list_restorable_tomes(
    app: AppHandle,
    session: State<'_, SessionState>,
) -> Result<Vec<RestorableTome>, String> {
    let active = session
        .current()
        .await
        .ok_or_else(|| "backup is locked — unlock it in Settings → Backup first".to_string())?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;

    let raw = build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
    let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();

    let summaries = crate::sync::snapshot::list_tome_snapshots(raw_arc.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(summaries.len());
    for s in summaries {
        let key = format!(
            "tomes/{}/snapshots/{}.snap.enc",
            s.tome_uuid, s.snapshot_id
        );
        // Restore to a temp file to peek metadata, then drop.
        let peek_dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
        let peek_path = peek_dir.path().join("peek.tome");
        if let Err(e) = crate::sync::snapshot::restore_snapshot_by_key(
            &key,
            &active.key,
            raw_arc.as_ref(),
            &peek_path,
        )
        .await
        {
            log::warn!("could not peek snapshot {key}: {e}");
            continue;
        }
        let (name, description) = match peek_tome_metadata(&peek_path).await {
            Ok(v) => v,
            Err(e) => {
                log::warn!("could not read metadata from {key}: {e}");
                (format!("Tome {}", &s.tome_uuid[..8]), None)
            }
        };
        out.push(RestorableTome {
            tome_uuid: s.tome_uuid,
            snapshot_id: s.snapshot_id,
            name,
            description,
            size_bytes: s.size_bytes,
            last_modified: s.last_modified.to_rfc3339(),
        });
    }
    Ok(out)
}

#[derive(Debug, Deserialize)]
pub struct RestoreInput {
    pub tome_uuid: String,
}

/// Download the latest snapshot for `tome_uuid`, decrypt, write to the
/// app data dir as a `.tome` file, register in recent_tomes. Returns the
/// new file path so the UI can immediately invoke `open_tome`.
#[tauri::command]
pub async fn backup_restore_tome(
    app: AppHandle,
    session: State<'_, SessionState>,
    input: RestoreInput,
) -> Result<RestoredTome, String> {
    let active = session
        .current()
        .await
        .ok_or_else(|| "backup is locked".to_string())?;
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;
    let raw = build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
    let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();

    let summaries = crate::sync::snapshot::list_tome_snapshots(raw_arc.as_ref())
        .await
        .map_err(|e| e.to_string())?;
    let summary = summaries
        .into_iter()
        .find(|s| s.tome_uuid == input.tome_uuid)
        .ok_or_else(|| format!("no snapshots for tome_uuid {}", input.tome_uuid))?;

    let key = format!(
        "tomes/{}/snapshots/{}.snap.enc",
        summary.tome_uuid, summary.snapshot_id
    );

    // Stage to a temp file first so a partial download never lands in
    // the user-visible Tomes directory.
    let stage_dir = app_data_dir.join("restored-staging");
    tokio::fs::create_dir_all(&stage_dir)
        .await
        .map_err(|e| format!("stage dir: {e}"))?;
    let stage_path = stage_dir.join(format!("{}.tome", summary.tome_uuid));
    crate::sync::snapshot::restore_snapshot_by_key(
        &key,
        &active.key,
        raw_arc.as_ref(),
        &stage_path,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("crypto") {
            "wrong passphrase — could not decrypt snapshot".to_string()
        } else {
            e.to_string()
        }
    })?;

    let (name, description) = peek_tome_metadata(&stage_path)
        .await
        .map_err(|e| format!("reading restored tome metadata: {e}"))?;

    // Move into a stable, user-visible location: <app_data>/restored/<safe_name>.tome
    let restored_dir = app_data_dir.join("restored");
    tokio::fs::create_dir_all(&restored_dir)
        .await
        .map_err(|e| format!("restored dir: {e}"))?;
    let safe_name = sanitize_filename(&name);
    let mut final_path = restored_dir.join(format!("{safe_name}.tome"));
    let mut suffix = 1;
    while final_path.exists() {
        final_path = restored_dir.join(format!("{safe_name} ({suffix}).tome"));
        suffix += 1;
    }
    tokio::fs::rename(&stage_path, &final_path)
        .await
        .map_err(|e| format!("moving restored tome into place: {e}"))?;

    let path_str = final_path.to_string_lossy().to_string();

    // Auto-enable sync on the restored Tome. A restored snapshot carries
    // no sync_config row (snapshots strip sync state intentionally) — but
    // the user explicitly chose to restore from backup, so they clearly
    // want this Tome to resume syncing. Seed a sync_config row with this
    // device's identity so the runner picks it up on next tick.
    let app_data_dir_for_cfg = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let app_cfg = app_backend::load(&app_data_dir_for_cfg)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;
    {
        use sqlx::sqlite::SqlitePoolOptions;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&format!("sqlite:{}", final_path.display()))
            .await
            .map_err(|e| format!("opening restored tome to enable sync: {e}"))?;
        let now = chrono::Utc::now();
        let cfg = crate::sync::state::SyncConfig {
            tome_id: path_str.clone(),
            enabled: true,
            device_id: app_cfg.device_id,
            created_at: now,
            updated_at: now,
        };
        cfg.save(&pool).await.map_err(|e| e.to_string())?;
        pool.close().await;
    }

    crate::app_state::add_recent_tome(&app, &path_str, &name, description.as_deref());

    Ok(RestoredTome {
        path: path_str,
        name,
        tome_uuid: summary.tome_uuid,
    })
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
            let raw =
                build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
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

async fn peek_tome_metadata(path: &std::path::Path) -> Result<(String, Option<String>), String> {
    use sqlx::sqlite::SqlitePoolOptions;
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", path.display()))
        .await
        .map_err(|e| format!("opening restored tome: {e}"))?;
    let name: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'name'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("reading name: {e}"))?;
    let description: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'description'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| format!("reading description: {e}"))?;
    pool.close().await;
    Ok((name.unwrap_or_else(|| "Untitled Tome".to_string()), description))
}

fn sanitize_filename(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "Restored Tome".to_string()
    } else {
        trimmed.to_string()
    }
}
