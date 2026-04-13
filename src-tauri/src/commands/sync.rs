//! M7 Sync — public Tauri commands.
//!
//! Replaces the dev-only `sync_dev_filesystem` from Phase 2. These are the
//! commands the Settings → Sync UI calls.

use crate::db::{self, ManagedDb};
use crate::sync::backend::filesystem::FilesystemBackend;
use crate::sync::backend::s3::{S3Backend, S3Config};
use crate::sync::backend::SyncBackend;
use crate::sync::crypto::{generate_salt, KeyMaterial};
use crate::sync::engine::sync_tome_once;
use crate::sync::session::{SessionState, SyncSession};
use crate::sync::state::{BackendKind, SyncConfig, SyncRuntimeState};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct SyncStatusPayload {
    /// `sync_config.enabled = 1` in the local DB. Persists across app restarts.
    pub enabled: bool,
    /// True when sync is `enabled` but the in-memory key is missing
    /// (after Tome reopen / app restart). User must call `sync_unlock` with
    /// the passphrase before the runner can sync.
    pub locked: bool,
    pub tome_id: Option<String>,
    pub backend_kind: Option<String>,
    pub backend_summary: Option<String>,
    pub device_name: Option<String>,
    pub last_sync_at: Option<String>,
    pub last_error: Option<String>,
    pub queue_size: i64,
    pub pending_conflicts: i64,
}

#[derive(Debug, Serialize)]
pub struct ConflictPayload {
    pub conflict_id: String,
    pub table_name: String,
    pub row_id: String,
    pub field_name: String,
    pub local_value: Option<String>,  // JSON-encoded
    pub remote_value: Option<String>, // JSON-encoded
    pub local_op_id: String,
    pub remote_op_id: String,
    pub detected_at: String,
}

#[derive(Debug, Deserialize)]
pub struct EnableSyncInput {
    pub tome_id: String,
    pub backend_kind: String, // "filesystem" | "s3"
    pub backend_config: serde_json::Value,
    pub passphrase: String,
    pub device_name: Option<String>,
}

#[tauri::command]
pub async fn sync_enable(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    input: EnableSyncInput,
) -> Result<SyncStatusPayload, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let backend_kind = BackendKind::from_str(&input.backend_kind)
        .ok_or_else(|| format!("unknown backend_kind: {}", input.backend_kind))?;

    // Validate backend config can be opened (catches bad paths / bad creds early).
    let _backend: Box<dyn SyncBackend + Send + Sync> =
        build_backend(backend_kind, &input.backend_config)
            .await
            .map_err(|e| format!("backend init failed: {e}"))?;

    // Find or create sync_config (idempotent re-enable).
    let existing = SyncConfig::load(&pool, &input.tome_id)
        .await
        .map_err(|e| e.to_string())?;
    let (salt, device_id) = match existing {
        Some(cfg) => (cfg.passphrase_salt, cfg.device_id),
        None => (generate_salt().to_vec(), Uuid::new_v4()),
    };

    let key = KeyMaterial::derive(&input.passphrase, &salt).map_err(|e| e.to_string())?;

    // Validate the passphrase against existing backend data. If the bucket
    // already has snapshots or journal entries from a previous sync session,
    // try to decrypt one with the derived key. A decrypt failure means the
    // user typed the wrong passphrase (or pointed at the wrong bucket).
    // Without this check, a wrong-passphrase re-enable silently accepts and
    // produces "wrong passphrase or tampered ciphertext" errors at sync time
    // — confusing UX.
    {
        let backend = build_backend(backend_kind, &input.backend_config)
            .await
            .map_err(|e| format!("backend init failed: {e}"))?;
        let existing_snaps = backend
            .list_prefix("snapshots")
            .await
            .map_err(|e| e.to_string())?;
        let existing_ops = backend
            .list_prefix("journal")
            .await
            .map_err(|e| e.to_string())?;
        let probe = existing_snaps.first().or(existing_ops.first());
        if let Some(obj) = probe {
            let (ciphertext, _etag) = backend
                .get_object(&obj.key)
                .await
                .map_err(|e| e.to_string())?;
            crate::sync::crypto::decrypt(&key, &ciphertext).map_err(|_| {
                "passphrase does not match the existing backend data — \
                 either the passphrase is wrong, or this bucket/folder \
                 already contains data from a different sync session"
                    .to_string()
            })?;
        }
    }

    let device_name = input
        .device_name
        .unwrap_or_else(|| std::env::var("HOSTNAME").unwrap_or_else(|_| "Vaelorium Device".into()));

    let cfg = SyncConfig {
        tome_id: input.tome_id.clone(),
        enabled: true,
        backend_type: backend_kind,
        backend_config: input.backend_config.clone(),
        passphrase_salt: salt,
        device_id,
        device_name: device_name.clone(),
        schema_version: 1,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    cfg.save(&pool).await.map_err(|e| e.to_string())?;

    // Cache the unlocked key for the runner.
    session
        .set(SyncSession {
            tome_id: input.tome_id.clone(),
            device_id,
            key: Arc::new(key),
            backend_kind,
            backend_config: input.backend_config,
        })
        .await;
    session.nudge();

    // Persist passphrase to OS keychain so the next Tome reopen can
    // auto-unlock without prompting. Best-effort: if no keychain backend
    // available (e.g. WSL without gnome-keyring), log and continue —
    // user will just have to re-enter on next launch.
    if let Err(e) = crate::sync::keychain::store(&input.tome_id, &input.passphrase) {
        log::warn!("could not store passphrase in OS keychain: {e}");
    }

    sync_status(managed, session).await
}

#[tauri::command]
pub async fn sync_disable(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    tome_id: String,
) -> Result<SyncStatusPayload, String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("UPDATE sync_config SET enabled = 0, updated_at = ? WHERE tome_id = ?")
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(&tome_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    session.clear().await;
    // Forget the keychain entry so the next Enable starts clean.
    if let Err(e) = crate::sync::keychain::delete(&tome_id) {
        log::warn!("could not delete keychain entry: {e}");
    }
    sync_status(managed, session).await
}

#[tauri::command]
pub async fn sync_now(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<crate::sync::SyncOutcome, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let active = session
        .current()
        .await
        .ok_or_else(|| "sync is not enabled".to_string())?;

    let backend = build_backend(active.backend_kind, &active.backend_config)
        .await
        .map_err(|e| format!("backend init failed: {e}"))?;

    sync_tome_once(&pool, &active.tome_id, &active.key, backend.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_status(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<SyncStatusPayload, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let active = session.current().await;
    let cfg_row: Option<(String, i64, String, String, String)> = sqlx::query_as(
        "SELECT tome_id, enabled, backend_type, backend_config, device_name FROM sync_config LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (enabled, tome_id, backend_kind, backend_summary, device_name) = match &cfg_row {
        Some((id, en, kind, cfg_json, dev)) => {
            let summary = match kind.as_str() {
                "filesystem" => {
                    let v: serde_json::Value =
                        serde_json::from_str(cfg_json).unwrap_or(json!({}));
                    v.get("path")
                        .and_then(|x| x.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_default()
                }
                _ => kind.clone(),
            };
            (
                *en != 0,
                Some(id.clone()),
                Some(kind.clone()),
                Some(summary),
                Some(dev.clone()),
            )
        }
        None => (false, None, None, None, None),
    };
    // "locked" = configured-and-enabled in DB but no in-memory key cached.
    // Happens after Tome reopen since the key derivation is process-local.
    let locked = enabled && active.is_none();

    let (last_sync_at, last_error, queue_size) = match &tome_id {
        Some(id) => {
            let st = SyncRuntimeState::load(&pool, id)
                .await
                .map_err(|e| e.to_string())?;
            // Count only ops not yet uploaded. sync_journal_local retains
            // already-uploaded ops until the next snapshot prunes them, so
            // a naive COUNT(*) overstates "pending uploads" badly.
            let qs: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM sync_journal_local
                 WHERE tome_id = ? AND op_id > COALESCE(?, '')",
            )
            .bind(id)
            .bind(st.last_uploaded_op_id.as_deref())
            .fetch_one(&pool)
            .await
            .unwrap_or(0);
            (st.last_sync_at.map(|t| t.to_rfc3339()), st.last_error, qs)
        }
        None => (None, None, 0),
    };

    let pending_conflicts: i64 = match &tome_id {
        Some(id) => sqlx::query_scalar("SELECT COUNT(*) FROM sync_conflicts WHERE tome_id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap_or(0),
        None => 0,
    };

    Ok(SyncStatusPayload {
        enabled,
        locked,
        tome_id,
        backend_kind,
        backend_summary,
        device_name,
        last_sync_at,
        last_error,
        queue_size,
        pending_conflicts,
    })
}

#[tauri::command]
pub async fn sync_take_snapshot(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<String, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let active = session
        .current()
        .await
        .ok_or_else(|| "sync is not enabled".to_string())?;
    let backend = build_backend(active.backend_kind, &active.backend_config)
        .await
        .map_err(|e| e.to_string())?;
    let info = crate::sync::snapshot::take_snapshot(&pool, &active.key, backend.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    // Persist as the new last_snapshot_id.
    let mut state = SyncRuntimeState::load(&pool, &active.tome_id)
        .await
        .map_err(|e| e.to_string())?;
    state.last_snapshot_id = Some(info.snapshot_id.to_string());
    state.save(&pool).await.map_err(|e| e.to_string())?;
    Ok(info.snapshot_id.to_string())
}

#[tauri::command]
pub async fn sync_list_conflicts(
    managed: State<'_, ManagedDb>,
) -> Result<Vec<ConflictPayload>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows: Vec<(String, String, String, String, Option<String>, Option<String>, String, String, String)> = sqlx::query_as(
        "SELECT conflict_id, table_name, row_id, field_name, local_value, remote_value, local_op_id, remote_op_id, detected_at FROM sync_conflicts ORDER BY detected_at",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|r| ConflictPayload {
            conflict_id: r.0,
            table_name: r.1,
            row_id: r.2,
            field_name: r.3,
            local_value: r.4,
            remote_value: r.5,
            local_op_id: r.6,
            remote_op_id: r.7,
            detected_at: r.8,
        })
        .collect())
}

#[derive(Debug, Deserialize)]
pub struct ResolveConflictInput {
    pub conflict_id: String,
    pub choose_local: bool,
}

#[tauri::command]
pub async fn sync_resolve_conflict(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    input: ResolveConflictInput,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Load conflict record.
    let row: Option<(String, String, String, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT tome_id, table_name, row_id, field_name, local_value, remote_value FROM sync_conflicts WHERE conflict_id = ?",
    )
    .bind(&input.conflict_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;
    let (tome_id, table, row_id, field, local_json, remote_json) =
        row.ok_or_else(|| format!("conflict {} not found", input.conflict_id))?;

    let chosen_json = if input.choose_local { local_json.clone() } else { remote_json.clone() };
    let chosen_value: Option<serde_json::Value> = chosen_json
        .as_ref()
        .map(|s| serde_json::from_str(s))
        .transpose()
        .map_err(|e| e.to_string())?;

    // Apply the chosen value to the local row (raw SQL; no op emission yet).
    let active = session.current().await;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let sql = format!("UPDATE {} SET {} = ? WHERE id = ?", table, field);
    let q = match &chosen_value {
        Some(v) if v.is_string() => sqlx::query(&sql).bind(v.as_str().unwrap()),
        Some(v) if v.is_i64() => sqlx::query(&sql).bind(v.as_i64().unwrap()),
        Some(v) if v.is_boolean() => sqlx::query(&sql).bind(v.as_bool().unwrap()),
        Some(_) => return Err(format!("unsupported value type for {field}")),
        None => sqlx::query(&sql).bind(Option::<String>::None),
    };
    q.bind(&row_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;

    // Emit a resolution op so the choice propagates to other devices.
    if let Some(active) = active {
        use crate::sync::journal::{record_op, update_op};
        use std::collections::BTreeMap;
        let mut after = BTreeMap::new();
        after.insert(field.clone(), chosen_value.clone());
        let mut before = BTreeMap::new();
        // We don't have the true "before" here; use the rejected side as before.
        let rejected_json = if input.choose_local { remote_json } else { local_json };
        let rejected_value: Option<serde_json::Value> = rejected_json
            .as_ref()
            .map(|s| serde_json::from_str(s))
            .transpose()
            .map_err(|e| e.to_string())?;
        before.insert(field.clone(), rejected_value);

        if let Some(op) = update_op(active.device_id, Ulid::new(), &table, &row_id, &before, &after) {
            record_op(&mut *tx, &op, &tome_id).await.map_err(|e| e.to_string())?;
        }
    }

    sqlx::query("DELETE FROM sync_conflicts WHERE conflict_id = ?")
        .bind(&input.conflict_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| e.to_string())?;

    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();
    Ok(())
}

/// Shared backend constructor used by sync_enable / sync_now / sync_take_snapshot
/// and by the runner (imported from there). Returns a boxed trait object so
/// callers don't need to know which concrete type to hold.
pub async fn build_backend(
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
                S3Backend::new(s3_cfg)
                    .await
                    .map_err(|e| e.to_string())?,
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

/// Restore the cached key from sync_config + a passphrase (called on app
/// startup if a Tome with sync enabled is opened).
#[tauri::command]
pub async fn sync_unlock(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    passphrase: String,
) -> Result<SyncStatusPayload, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let cfg = sqlx::query_as::<_, (String, i64, String, String, Vec<u8>, String)>(
        "SELECT tome_id, enabled, backend_type, backend_config, passphrase_salt, device_id FROM sync_config LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "sync not configured for this Tome".to_string())?;

    let backend_kind = BackendKind::from_str(&cfg.2)
        .ok_or_else(|| format!("unknown backend kind {}", cfg.2))?;
    let backend_config: serde_json::Value =
        serde_json::from_str(&cfg.3).map_err(|e| e.to_string())?;
    let key = KeyMaterial::derive(&passphrase, &cfg.4).map_err(|e| e.to_string())?;
    let device_id = Uuid::parse_str(&cfg.5).map_err(|e| e.to_string())?;

    // Validate the passphrase against existing backend data BEFORE caching.
    // Without this, a wrong passphrase silently caches a bogus key, the
    // sidebar pill briefly shows green, and the runner only fails ~10s later
    // when it tries to apply remote ops. Confusing UX.
    let backend = build_backend(backend_kind, &backend_config)
        .await
        .map_err(|e| format!("backend init failed: {e}"))?;
    let existing_snaps = backend
        .list_prefix("snapshots")
        .await
        .map_err(|e| e.to_string())?;
    let existing_ops = backend
        .list_prefix("journal")
        .await
        .map_err(|e| e.to_string())?;
    let probe = existing_snaps.first().or(existing_ops.first());
    if let Some(obj) = probe {
        let (ciphertext, _etag) = backend
            .get_object(&obj.key)
            .await
            .map_err(|e| e.to_string())?;
        crate::sync::crypto::decrypt(&key, &ciphertext).map_err(|_| {
            "wrong passphrase — could not decrypt existing sync data".to_string()
        })?;
    }
    // (If backend has no data yet, the unlock is trivially valid — there's
    // nothing to decrypt-check against. First sync will write the first blob.)

    if cfg.1 != 0 {
        session
            .set(SyncSession {
                tome_id: cfg.0.clone(),
                device_id,
                key: Arc::new(key),
                backend_kind,
                backend_config,
            })
            .await;
        session.nudge();

        // Refresh the keychain entry so future auto-unlocks succeed.
        if let Err(e) = crate::sync::keychain::store(&cfg.0, &passphrase) {
            log::warn!("could not refresh keychain entry: {e}");
        }
    }
    sync_status(managed, session).await
}

/// Attempt to unlock sync using a passphrase stored in the OS keychain.
/// Returns true if a stored passphrase was found AND validated successfully.
/// Returns false in all other cases — caller falls back to manual unlock UI.
/// Never errors loudly: keychain unavailable, no entry, wrong stored
/// passphrase (rotated externally?) all return Ok(false).
#[tauri::command]
pub async fn sync_try_auto_unlock(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<bool, String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Already unlocked? Nothing to do.
    if session.current().await.is_some() {
        return Ok(true);
    }

    let cfg: Option<(String, i64)> =
        sqlx::query_as("SELECT tome_id, enabled FROM sync_config LIMIT 1")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;
    let Some((tome_id, enabled)) = cfg else { return Ok(false) };
    if enabled == 0 {
        return Ok(false);
    }

    let passphrase = match crate::sync::keychain::retrieve(&tome_id) {
        Ok(Some(p)) => p,
        _ => return Ok(false),
    };

    // Reuse sync_unlock's path. If the stored passphrase is wrong (e.g.
    // user changed it externally and sync_unlock validates it against
    // backend data), the call returns Err — surface as Ok(false) so the
    // UI shows the manual unlock prompt.
    Ok(sync_unlock(managed, session, passphrase).await.is_ok())
}
