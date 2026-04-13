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

/// Returns true when VAELORIUM_WIZARD env var is set (any non-empty value).
/// Used to force-open the first-run backup wizard for testing without
/// disconnecting the real backup.
#[tauri::command]
pub async fn should_show_wizard_override() -> Result<bool, String> {
    Ok(std::env::var("VAELORIUM_WIZARD").map(|v| !v.is_empty()).unwrap_or(false))
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

    // Set as active pool
    {
        let mut guard = managed.write().await;
        *guard = Some(pool);
    }

    // Add to recent tomes
    app_state::add_recent_tome(&app, &path, &name, description.as_deref());

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

    // Set as active pool
    {
        let mut guard = managed.write().await;
        *guard = Some(pool);
    }

    // Add to recent tomes
    app_state::add_recent_tome(&app, &path, &name, description.as_deref());

    Ok(TomeInfo {
        path,
        name,
        description,
    })
}

#[tauri::command]
pub async fn close_tome(managed: State<'_, ManagedDb>) -> Result<(), String> {
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
