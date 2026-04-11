use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageInfo {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageData {
    pub id: String,
    pub filename: String,
    pub mime_type: String,
    pub data: Vec<u8>,
}

fn mime_from_extension(filename: &str) -> &str {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
pub async fn upload_image(
    managed: State<'_, ManagedDb>,
    path: String,
) -> Result<ImageInfo, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let file_data = std::fs::read(&path).map_err(|e| format!("Failed to read file: {}", e))?;

    let filename = std::path::Path::new(&path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image")
        .to_string();

    let mime_type = mime_from_extension(&filename).to_string();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO images (id, filename, mime_type, data, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&filename)
    .bind(&mime_type)
    .bind(&file_data)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ImageInfo {
        id,
        filename,
        mime_type,
        created_at: now,
    })
}

#[tauri::command]
pub async fn upload_image_data(
    managed: State<'_, ManagedDb>,
    filename: String,
    data: Vec<u8>,
) -> Result<ImageInfo, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let mime_type = mime_from_extension(&filename).to_string();
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    sqlx::query(
        "INSERT INTO images (id, filename, mime_type, data, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&filename)
    .bind(&mime_type)
    .bind(&data)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(ImageInfo {
        id,
        filename,
        mime_type,
        created_at: now,
    })
}

#[tauri::command]
pub async fn get_image(
    managed: State<'_, ManagedDb>,
    id: String,
) -> Result<ImageData, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let row = sqlx::query_as::<_, (String, String, String, Vec<u8>)>(
        "SELECT id, filename, mime_type, data FROM images WHERE id = ?",
    )
    .bind(&id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "Image not found".to_string())?;

    Ok(ImageData {
        id: row.0,
        filename: row.1,
        mime_type: row.2,
        data: row.3,
    })
}

#[tauri::command]
pub async fn delete_image(
    managed: State<'_, ManagedDb>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;

    sqlx::query("DELETE FROM images WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn list_images(
    managed: State<'_, ManagedDb>,
) -> Result<Vec<ImageInfo>, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let rows = sqlx::query_as::<_, (String, String, String, String)>(
        "SELECT id, filename, mime_type, created_at FROM images ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| ImageInfo {
            id: row.0,
            filename: row.1,
            mime_type: row.2,
            created_at: row.3,
        })
        .collect())
}
