//! Per-Tome sync commands.
//!
//! Backup destination + passphrase are app-global (see `commands::backup`).
//! These commands just toggle whether the active Tome participates in
//! sync and report its status. The runner picks up enabled Tomes
//! automatically.

use crate::commands::backup as backup_cmd;
use crate::db::{self, ManagedDb};
use crate::sync::app_backend;
use crate::sync::backend::prefixed::{tome_prefix, PrefixedBackend};
use crate::sync::backend::SyncBackend;
use crate::sync::engine::sync_tome_once;
use crate::sync::session::SessionState;
use crate::sync::state::{SyncConfig, SyncRuntimeState};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use ulid::Ulid;

#[derive(Debug, Serialize)]
pub struct SyncStatusPayload {
    /// Per-Tome enabled flag (sync_config.enabled).
    pub enabled: bool,
    /// True if (backup configured AND tome enabled) but the app-global key
    /// isn't cached yet (after app restart, before passphrase entry).
    pub locked: bool,
    /// True if the user hasn't configured a backup destination at all.
    pub backup_missing: bool,
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
    pub local_value: Option<String>,
    pub remote_value: Option<String>,
    pub local_op_id: String,
    pub remote_op_id: String,
    pub detected_at: String,
    /// Human-readable label for the conflicting row (e.g. a page title or
    /// entity-type name). Falls back to a short row_id if the row is no
    /// longer in the local DB or the table has no natural name column.
    pub row_label: String,
    /// Humanized table name for UI ("Page", "Entity type", ...).
    pub table_label: String,
}

fn humanize_table(table: &str) -> &'static str {
    match table {
        "pages" => "Page",
        "entity_types" => "Entity type",
        "entity_type_fields" => "Entity type field",
        "entity_field_values" => "Entity field value",
        "relations" => "Relation",
        "relation_types" => "Relation type",
        "tags" => "Tag",
        "page_tags" => "Page tag",
        "maps" => "Map",
        "map_pins" => "Map pin",
        "timelines" => "Timeline",
        "timeline_events" => "Timeline event",
        "boards" => "Board",
        "board_cards" => "Board card",
        "board_connectors" => "Board connector",
        _ => "Row",
    }
}

/// Best-effort resolution of a row's human-readable label. The column
/// choice per table is hard-coded — only known tables get a join; anything
/// else falls back to a truncated row_id. Safe against SQL injection
/// because both `table` and `col` are selected from a fixed match.
async fn resolve_row_label(pool: &sqlx::SqlitePool, table: &str, row_id: &str) -> String {
    let (lookup_col, name_col): (&str, Option<&str>) = match table {
        "pages" => ("id", Some("title")),
        "entity_types" => ("id", Some("name")),
        "entity_type_fields" => ("id", Some("label")),
        "tags" => ("id", Some("name")),
        "maps" => ("id", Some("name")),
        "timelines" => ("id", Some("name")),
        "boards" => ("id", Some("name")),
        "relation_types" => ("id", Some("name")),
        _ => ("id", None),
    };
    if let Some(col) = name_col {
        let sql = format!("SELECT {col} FROM {table} WHERE {lookup_col} = ? LIMIT 1");
        if let Ok(Some(name)) = sqlx::query_scalar::<_, Option<String>>(&sql)
            .bind(row_id)
            .fetch_optional(pool)
            .await
        {
            if let Some(n) = name.filter(|s| !s.is_empty()) {
                return n;
            }
        }
    }
    // Fallback: short prefix so the user has *something* to distinguish rows.
    let short: String = row_id.chars().take(8).collect();
    format!("({short}…)")
}

#[derive(Debug, Deserialize)]
pub struct EnableSyncInput {
    pub tome_id: String,
}

#[tauri::command]
pub async fn sync_enable(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    input: EnableSyncInput,
) -> Result<SyncStatusPayload, String> {
    // Backup destination must already be configured + unlocked.
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let app_cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured — set one up in Settings → Backup first".to_string())?;
    if session.current().await.is_none() {
        return Err("backup is locked — unlock it in Settings → Backup first".to_string());
    }

    let pool = db::get_pool(managed.inner()).await?;
    let now = chrono::Utc::now();

    // Materialize the Tome's stable UUID before any op ships, so the
    // bucket prefix is locked in from the very first upload.
    crate::sync::tome_identity::get_or_create_uuid(&pool)
        .await
        .map_err(|e| format!("tome identity: {e}"))?;

    let existing = SyncConfig::load(&pool, &input.tome_id)
        .await
        .map_err(|e| e.to_string())?;
    // Stamp the app-global device_id into sync_config so mutation paths
    // that call `active_sync_session(pool)` pick up the same attribution.
    let cfg = SyncConfig {
        tome_id: input.tome_id.clone(),
        enabled: true,
        device_id: app_cfg.device_id,
        created_at: existing.as_ref().map(|c| c.created_at).unwrap_or(now),
        updated_at: now,
    };
    cfg.save(&pool).await.map_err(|e| e.to_string())?;
    session.nudge();
    sync_status(app, managed, session).await
}

#[tauri::command]
pub async fn sync_disable(
    app: AppHandle,
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
    sync_status(app, managed, session).await
}

#[tauri::command]
pub async fn sync_now(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<crate::sync::SyncOutcome, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let active = session
        .current()
        .await
        .ok_or_else(|| "backup is locked".to_string())?;

    // Find the enabled Tome (single-Tome path for now; runner iterates).
    let tome = SyncConfig::list_all(&pool)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|c| c.enabled)
        .ok_or_else(|| "sync is not enabled for any Tome".to_string())?;

    let backend = build_tome_backend(&app, &pool).await?;
    sync_tome_once(&pool, &tome.tome_id, &active.key, backend.as_ref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_status(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<SyncStatusPayload, String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Backup status (app-global).
    let backup = backup_cmd::backup_status(app.clone(), session.clone()).await?;

    // Per-Tome status.
    let cfg_row: Option<(String, i64)> = sqlx::query_as(
        "SELECT tome_id, enabled FROM sync_config LIMIT 1",
    )
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    let (tome_enabled, tome_id) = match &cfg_row {
        Some((id, en)) => (*en != 0, Some(id.clone())),
        None => (false, None),
    };
    // device_name now lives on backup_status (app-global).
    let device_name = backup.device_name.clone();

    // "locked" surfaces whenever there's something to unlock — either
    // backup-locked OR a per-Tome that wants to sync but the key is gone.
    let locked = backup.locked || (tome_enabled && session.current().await.is_none());

    let (last_sync_at, last_error, queue_size) = match &tome_id {
        Some(id) => {
            let st = SyncRuntimeState::load(&pool, id)
                .await
                .map_err(|e| e.to_string())?;
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
        enabled: tome_enabled,
        locked,
        backup_missing: !backup.configured,
        tome_id,
        backend_kind: backup.backend_kind,
        backend_summary: backup.backend_summary,
        device_name,
        last_sync_at,
        last_error,
        queue_size,
        pending_conflicts,
    })
}

#[tauri::command]
pub async fn sync_take_snapshot(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
) -> Result<String, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let active = session
        .current()
        .await
        .ok_or_else(|| "backup is locked".to_string())?;
    let tome = SyncConfig::list_all(&pool)
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|c| c.enabled)
        .ok_or_else(|| "sync is not enabled for any Tome".to_string())?;
    let backend = build_tome_backend(&app, &pool).await?;
    let info = crate::sync::snapshot::take_snapshot(&pool, &active.key, backend.as_ref())
        .await
        .map_err(|e| e.to_string())?;

    let mut state = SyncRuntimeState::load(&pool, &tome.tome_id)
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

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        let row_label = resolve_row_label(&pool, &r.1, &r.2).await;
        let table_label = humanize_table(&r.1).to_string();
        out.push(ConflictPayload {
            conflict_id: r.0,
            table_name: r.1,
            row_id: r.2,
            field_name: r.3,
            local_value: r.4,
            remote_value: r.5,
            local_op_id: r.6,
            remote_op_id: r.7,
            detected_at: r.8,
            row_label,
            table_label,
        });
    }
    Ok(out)
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
    if session.current().await.is_some() {
        let device_id = SyncConfig::load(&pool, &tome_id)
            .await
            .map_err(|e| e.to_string())?
            .map(|c| c.device_id);
        if let Some(device_id) = device_id {
            use crate::sync::journal::{record_op, update_op};
            use std::collections::BTreeMap;
            let mut after = BTreeMap::new();
            after.insert(field.clone(), chosen_value.clone());
            let mut before = BTreeMap::new();
            let rejected_json = if input.choose_local { remote_json } else { local_json };
            let rejected_value: Option<serde_json::Value> = rejected_json
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .map_err(|e| e.to_string())?;
            before.insert(field.clone(), rejected_value);

            if let Some(op) = update_op(device_id, Ulid::new(), &table, &row_id, &before, &after) {
                record_op(&mut *tx, &op, &tome_id).await.map_err(|e| e.to_string())?;
            }
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

/// Build a Tome-prefixed backend by loading the app-global config and
/// wrapping with `tomes/{tome_uuid}/`. The UUID is resolved from the
/// per-Tome `tome_metadata` table (lazy-created on first access) so the
/// prefix is device-independent. Used by sync_now / sync_take_snapshot
/// and the runner.
pub async fn build_tome_backend(
    app: &AppHandle,
    pool: &sqlx::SqlitePool,
) -> Result<Box<dyn SyncBackend + Send + Sync>, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))?;
    let cfg = app_backend::load(&app_data_dir)
        .await?
        .ok_or_else(|| "no backup destination configured".to_string())?;
    let raw = backup_cmd::build_raw_backend(cfg.backend_kind, &cfg.backend_config).await?;
    let raw_arc: Arc<dyn SyncBackend + Send + Sync> = raw.into();
    let uuid = crate::sync::tome_identity::get_or_create_uuid(pool)
        .await
        .map_err(|e| format!("tome identity: {e}"))?;
    Ok(Box::new(PrefixedBackend::new(raw_arc, tome_prefix(&uuid))))
}
