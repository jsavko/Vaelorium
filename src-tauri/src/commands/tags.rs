use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[tauri::command]
pub async fn create_tag(
    managed: State<'_, ManagedDb>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO tags (id, name, color) VALUES (?, ?, ?)")
        .bind(&id)
        .bind(&name)
        .bind(&color)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Tag { id, name, color })
}

#[tauri::command]
pub async fn list_tags(managed: State<'_, ManagedDb>) -> Result<Vec<Tag>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT id, name, color FROM tags ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| Tag {
            id: row.0,
            name: row.1,
            color: row.2,
        })
        .collect())
}

#[tauri::command]
pub async fn add_tag_to_page(
    managed: State<'_, ManagedDb>,
    page_id: String,
    tag_id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("INSERT OR IGNORE INTO page_tags (page_id, tag_id) VALUES (?, ?)")
        .bind(&page_id)
        .bind(&tag_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_page(
    managed: State<'_, ManagedDb>,
    page_id: String,
    tag_id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("DELETE FROM page_tags WHERE page_id = ? AND tag_id = ?")
        .bind(&page_id)
        .bind(&tag_id)
        .execute(&pool)
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_page_tags(
    managed: State<'_, ManagedDb>,
    page_id: String,
) -> Result<Vec<Tag>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT t.id, t.name, t.color FROM tags t
         JOIN page_tags pt ON pt.tag_id = t.id
         WHERE pt.page_id = ?
         ORDER BY t.name",
    )
    .bind(&page_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| Tag {
            id: row.0,
            name: row.1,
            color: row.2,
        })
        .collect())
}
