use crate::app_state;
use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomeInfo {
    pub path: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TomeMetadata {
    pub name: String,
    pub description: Option<String>,
    pub cover_image: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn get_app_state(app: AppHandle) -> Result<app_state::AppState, String> {
    Ok(app_state::load_app_state(&app))
}

#[tauri::command]
pub async fn create_tome(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    path: String,
    name: String,
    description: Option<String>,
) -> Result<TomeInfo, String> {
    // Create and open the database
    let pool = db::open_database(&path)
        .await
        .map_err(|e| format!("Failed to create tome: {}", e))?;

    // Seed tome_metadata
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT OR REPLACE INTO tome_metadata (key, value) VALUES ('name', ?)")
        .bind(&name)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(ref desc) = description {
        sqlx::query("INSERT OR REPLACE INTO tome_metadata (key, value) VALUES ('description', ?)")
            .bind(desc)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    sqlx::query("INSERT OR REPLACE INTO tome_metadata (key, value) VALUES ('created_at', ?)")
        .bind(&now)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // Materialize the stable per-Tome UUID at creation time so
    // RecentTome can carry it for backup cross-referencing.
    let tome_uuid = crate::sync::tome_identity::get_or_create_uuid(&pool)
        .await
        .map_err(|e| e.to_string())?;

    // Set as active pool
    {
        let mut guard = managed.write().await;
        *guard = Some(pool);
    }

    // Add to recent tomes
    app_state::add_recent_tome(&app, &path, &name, description.as_deref(), Some(&tome_uuid));

    Ok(TomeInfo {
        path,
        name,
        description,
    })
}

#[tauri::command]
pub async fn open_tome(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, crate::sync::session::SessionState>,
    path: String,
) -> Result<TomeInfo, String> {
    // Close existing pool if any
    {
        let mut guard = managed.write().await;
        *guard = None;
    }

    // Open the database
    let pool = db::open_database(&path)
        .await
        .map_err(|e| format!("Failed to open tome: {}", e))?;

    // Read tome metadata
    let name: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'name'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let description: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'description'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let name = name.unwrap_or_else(|| {
        // Derive name from filename
        std::path::Path::new(&path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled Tome")
            .to_string()
    });

    // Read the tome_uuid for recent-tomes cross-referencing.
    let tome_uuid = crate::sync::tome_identity::get_or_create_uuid(&pool)
        .await
        .ok();

    // Set as active pool
    {
        let mut guard = managed.write().await;
        *guard = Some(pool);
    }

    // Add to recent tomes
    app_state::add_recent_tome(&app, &path, &name, description.as_deref(), tome_uuid.as_deref());

    // Nudge the sync runner to pull any remote changes for this Tome
    // right away. Replaces the per-typing nudge model with discrete
    // "sync on open" + "sync on close" triggers plus the slow poll.
    session.nudge();

    Ok(TomeInfo {
        path,
        name,
        description,
    })
}

#[tauri::command]
pub async fn close_tome(
    app: AppHandle,
    managed: State<'_, ManagedDb>,
    session: State<'_, crate::sync::session::SessionState>,
) -> Result<(), String> {
    // Sync-on-close: run one sync cycle synchronously before clearing
    // the managed pool so the user's latest edits are flushed to
    // backend. Best-effort with a 30s timeout — we never want close
    // to hang, but we always want a shot at flushing.
    if let Some(pool) = managed.read().await.clone() {
        if let Some(active) = session.current().await {
            if let Ok(cfgs) = crate::sync::state::SyncConfig::list_all(&pool).await {
                if let Some(cfg) = cfgs.into_iter().find(|c| c.enabled) {
                    let app_clone = app.clone();
                    let tome_id = cfg.tome_id.clone();
                    let key = active.key.clone();
                    let pool_clone = pool.clone();
                    let fut = async move {
                        if let Ok(backend) =
                            crate::commands::sync::build_tome_backend(&app_clone, &pool_clone)
                                .await
                        {
                            let _ = crate::sync::sync_tome_once(
                                &pool_clone,
                                &tome_id,
                                &*key,
                                backend.as_ref(),
                            )
                            .await;
                        }
                    };
                    let _ =
                        tokio::time::timeout(std::time::Duration::from_secs(30), fut).await;
                }
            }
        }
    }

    let mut guard = managed.write().await;
    *guard = None;
    log::info!("Tome closed");
    Ok(())
}

#[tauri::command]
pub async fn get_tome_metadata(managed: State<'_, ManagedDb>) -> Result<TomeMetadata, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let name: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'name'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let description: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'description'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let cover_image: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'cover_image'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    let created_at: Option<String> =
        sqlx::query_scalar("SELECT value FROM tome_metadata WHERE key = 'created_at'")
            .fetch_optional(&pool)
            .await
            .map_err(|e| e.to_string())?;

    Ok(TomeMetadata {
        name: name.unwrap_or_else(|| "Untitled Tome".to_string()),
        description,
        cover_image,
        created_at: created_at.unwrap_or_default(),
    })
}

#[tauri::command]
pub async fn update_tome_metadata(
    managed: State<'_, ManagedDb>,
    key: String,
    value: Option<String>,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("INSERT OR REPLACE INTO tome_metadata (key, value) VALUES (?, ?)")
        .bind(&key)
        .bind(&value)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
