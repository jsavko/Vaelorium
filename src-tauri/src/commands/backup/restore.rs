//! Enumerate restorable Tomes + download a snapshot into a local `.tome`.

use super::config::build_raw_backend;
use super::{RestorableTome, RestoreInput, RestoredTome};
use crate::sync::app_backend;
use crate::sync::backend::SyncBackend;
use crate::sync::session::SessionState;
use crate::sync::state::BackendKind;
use chrono::TimeZone;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

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

    // Hosted uses the dedicated `GET /v1/tomes` endpoint instead of a
    // raw bucket scan — each Tome is URL-scoped, no shared prefix.
    if matches!(cfg.backend_kind, BackendKind::Hosted) {
        return list_hosted_restorable_tomes(&app, &active.key).await;
    }

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

    // Hosted vs raw-backend paths diverge: hosted uses the per-Tome
    // HostedBackend + a snapshot key relative to that backend's root;
    // Filesystem/S3 wrap a raw backend that sees absolute
    // `tomes/<uuid>/...` keys.
    let (backend_arc, snap_key) = if matches!(cfg.backend_kind, BackendKind::Hosted) {
        let token = crate::commands::cloud::require_device_token_with_app(&app)?;
        let http = crate::sync::backend::hosted::HostedClient::new().map_err(|e| e.to_string())?;
        let hosted = crate::sync::backend::hosted::HostedBackend::new(
            http,
            input.tome_uuid.clone(),
            token.clone(),
        );
        let b_arc: Arc<dyn SyncBackend + Send + Sync> = Arc::new(hosted);
        // Cloud's GET /v1/tomes gives us latest_snapshot_id directly;
        // call it rather than listing per-tome snapshots which is
        // extra round-trips for hosted.
        let client = crate::sync::backend::hosted::HostedClient::new().map_err(|e| e.to_string())?;
        let summaries = client
            .list_account_tomes(&token)
            .await
            .map_err(|e| e.to_string())?;
        let summary = summaries
            .into_iter()
            .find(|s| s.tome_uuid == input.tome_uuid)
            .ok_or_else(|| format!("no Tome with uuid {}", input.tome_uuid))?;
        let snap_id = summary
            .latest_snapshot_id
            .ok_or_else(|| "this Tome has no snapshot yet — nothing to restore".to_string())?;
        let key = format!("snapshots/{snap_id}.snap.enc");
        (b_arc, key)
    } else {
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
        (raw_arc, key)
    };

    // Stage to a temp file first so a partial download never lands in
    // the user-visible Tomes directory.
    let stage_dir = app_data_dir.join("restored-staging");
    tokio::fs::create_dir_all(&stage_dir)
        .await
        .map_err(|e| format!("stage dir: {e}"))?;
    let stage_path = stage_dir.join(format!("{}.tome", input.tome_uuid));
    crate::sync::snapshot::restore_snapshot_by_key(
        &snap_key,
        &active.key,
        backend_arc.as_ref(),
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

    crate::app_state::add_recent_tome(&app, &path_str, &name, description.as_deref(), Some(&input.tome_uuid), true);

    Ok(RestoredTome {
        path: path_str,
        name,
        tome_uuid: input.tome_uuid,
    })
}

/// Hosted-specific restore listing. Hits `GET /v1/tomes` for the
/// tome_uuid + snapshot metadata, then downloads each tome's latest
/// snapshot through a per-tome `PrefixedBackend` wrapped around the
/// raw hosted client so we can peek `tome_metadata.name`. Mirrors
/// what Filesystem + S3 do via `snapshot::list_tome_snapshots` + peek.
async fn list_hosted_restorable_tomes(
    app: &AppHandle,
    key: &crate::sync::crypto::KeyMaterial,
) -> Result<Vec<RestorableTome>, String> {
    use crate::sync::backend::hosted::protocol::AccountTomeSummary;
    use std::sync::Arc as StdArc;

    let token = crate::commands::cloud::require_device_token_with_app(app)?;
    let client = crate::sync::backend::hosted::HostedClient::new().map_err(|e| e.to_string())?;
    let summaries: Vec<AccountTomeSummary> = client
        .list_account_tomes(&token)
        .await
        .map_err(|e| e.to_string())?;

    let mut out = Vec::with_capacity(summaries.len());
    for s in summaries {
        // Journal-only Tomes (no snapshot) can't be peeked for a
        // display name. Surface them with a fallback label so the user
        // at least sees them and can Delete if desired.
        let Some(snap_id) = s.latest_snapshot_id.clone() else {
            out.push(RestorableTome {
                tome_uuid: s.tome_uuid.clone(),
                snapshot_id: String::new(),
                name: format!("Tome {} (no snapshot yet)", &s.tome_uuid[..8]),
                description: None,
                size_bytes: s.size_bytes,
                last_modified: chrono::Utc
                    .timestamp_millis_opt(s.last_modified_ms)
                    .single()
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339(),
            });
            continue;
        };

        // Build a per-Tome HostedBackend so we can download the
        // snapshot through the trait. Skip a `get_object` direct path
        // to keep this uniform with the filesystem/S3 peek flow.
        let http =
            crate::sync::backend::hosted::HostedClient::new().map_err(|e| e.to_string())?;
        let hosted = crate::sync::backend::hosted::HostedBackend::new(
            http,
            s.tome_uuid.clone(),
            token.clone(),
        );
        let peek_dir = tempfile::tempdir().map_err(|e| format!("tempdir: {e}"))?;
        let peek_path = peek_dir.path().join("peek.tome");
        let snap_key = format!("snapshots/{snap_id}.snap.enc");
        // Use the engine's restore path which decrypts + writes atomically.
        let backend_dyn: StdArc<dyn crate::sync::backend::SyncBackend + Send + Sync> =
            StdArc::new(hosted);
        if let Err(e) = crate::sync::snapshot::restore_snapshot_by_key(
            &snap_key,
            key,
            backend_dyn.as_ref(),
            &peek_path,
        )
        .await
        {
            log::warn!("could not peek hosted snapshot {}: {}", s.tome_uuid, e);
            out.push(RestorableTome {
                tome_uuid: s.tome_uuid.clone(),
                snapshot_id: snap_id,
                name: format!("Tome {}", &s.tome_uuid[..8]),
                description: None,
                size_bytes: s.size_bytes,
                last_modified: chrono::Utc
                    .timestamp_millis_opt(s.last_modified_ms)
                    .single()
                    .unwrap_or_else(chrono::Utc::now)
                    .to_rfc3339(),
            });
            continue;
        }
        let (name, description) = peek_tome_metadata(&peek_path)
            .await
            .unwrap_or_else(|e| {
                log::warn!("could not read metadata from hosted {}: {}", s.tome_uuid, e);
                (format!("Tome {}", &s.tome_uuid[..8]), None)
            });
        out.push(RestorableTome {
            tome_uuid: s.tome_uuid,
            snapshot_id: snap_id,
            name,
            description,
            size_bytes: s.size_bytes,
            last_modified: chrono::Utc
                .timestamp_millis_opt(s.last_modified_ms)
                .single()
                .unwrap_or_else(chrono::Utc::now)
                .to_rfc3339(),
        });
    }
    Ok(out)
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
