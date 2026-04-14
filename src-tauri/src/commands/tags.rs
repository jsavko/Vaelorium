use crate::db::{self, ManagedDb};
use crate::sync::journal::{self, active_sync_session, delete_op, emit_for_row, insert_op, record_op};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}

#[tauri::command]
pub async fn create_tag(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    name: String,
    color: Option<String>,
) -> Result<Tag, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO tags (id, name, color) VALUES (?, ?, ?)")
        .bind(&id).bind(&name).bind(&color)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.tags, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

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
    _session: State<'_, SessionState>,
    page_id: String,
    tag_id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT OR IGNORE INTO page_tags (page_id, tag_id) VALUES (?, ?)")
        .bind(&page_id).bind(&tag_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    if let Some((tome_id, device_id)) = sync_session.as_ref() {
        let composite = format!("{}|{}", page_id, tag_id);
        let mut fields = BTreeMap::new();
        fields.insert("page_id".to_string(), Some(serde_json::json!(page_id)));
        fields.insert("tag_id".to_string(), Some(serde_json::json!(tag_id)));
        let op = insert_op(*device_id, Ulid::new(), "page_tags", &composite, fields);
        record_op(&mut *tx, &op, tome_id).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn remove_tag_from_page(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    page_id: String,
    tag_id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("DELETE FROM page_tags WHERE page_id = ? AND tag_id = ?")
        .bind(&page_id).bind(&tag_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    if let Some((tome_id, device_id)) = sync_session.as_ref() {
        let composite = format!("{}|{}", page_id, tag_id);
        let mut prev = BTreeMap::new();
        prev.insert("page_id".to_string(), Some(serde_json::json!(page_id)));
        prev.insert("tag_id".to_string(), Some(serde_json::json!(tag_id)));
        let op = delete_op(*device_id, Ulid::new(), "page_tags", &composite, prev);
        record_op(&mut *tx, &op, tome_id).await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
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
