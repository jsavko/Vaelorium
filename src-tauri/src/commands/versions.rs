use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct PageVersion {
    pub id: String,
    pub page_id: String,
    pub version_number: i64,
    pub created_at: String,
    pub created_by: Option<String>,
    pub summary: Option<String>,
}

#[tauri::command]
pub async fn create_version(
    pool: State<'_, DbPool>,
    page_id: String,
    yjs_snapshot: Vec<u8>,
    summary: Option<String>,
) -> Result<PageVersion, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let max_version: Option<i64> = sqlx::query_scalar(
        "SELECT MAX(version_number) FROM page_versions WHERE page_id = ?",
    )
    .bind(&page_id)
    .fetch_one(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    let version_number = max_version.unwrap_or(0) + 1;

    sqlx::query(
        "INSERT INTO page_versions (id, page_id, yjs_snapshot, version_number, created_at, summary)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&id)
    .bind(&page_id)
    .bind(&yjs_snapshot)
    .bind(version_number)
    .bind(&now)
    .bind(&summary)
    .execute(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(PageVersion {
        id,
        page_id,
        version_number,
        created_at: now,
        created_by: None,
        summary,
    })
}

#[tauri::command]
pub async fn list_versions(
    pool: State<'_, DbPool>,
    page_id: String,
) -> Result<Vec<PageVersion>, String> {
    let rows = sqlx::query_as::<_, (String, String, i64, String, Option<String>, Option<String>)>(
        "SELECT id, page_id, version_number, created_at, created_by, summary
         FROM page_versions WHERE page_id = ? ORDER BY version_number DESC",
    )
    .bind(&page_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| PageVersion {
            id: row.0,
            page_id: row.1,
            version_number: row.2,
            created_at: row.3,
            created_by: row.4,
            summary: row.5,
        })
        .collect())
}

#[tauri::command]
pub async fn get_version_snapshot(
    pool: State<'_, DbPool>,
    version_id: String,
) -> Result<Vec<u8>, String> {
    let result: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT yjs_snapshot FROM page_versions WHERE id = ?",
    )
    .bind(&version_id)
    .fetch_optional(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(result.unwrap_or_default())
}
