use crate::db::{self, ManagedDb};
use crate::sync::journal::{self, active_sync_session, emit_for_row, load_row_via_schema};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FieldValue {
    pub id: String,
    pub page_id: String,
    pub field_id: String,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PageByFieldResult {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub entity_type_id: Option<String>,
}

#[tauri::command]
pub async fn get_page_field_values(
    managed: State<'_, ManagedDb>,
    page_id: String,
) -> Result<Vec<FieldValue>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>)>(
        "SELECT id, page_id, field_id, value FROM entity_field_values WHERE page_id = ?",
    )
    .bind(&page_id)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| FieldValue {
            id: row.0,
            page_id: row.1,
            field_id: row.2,
            value: row.3,
        })
        .collect())
}

#[tauri::command]
pub async fn set_field_value(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    page_id: String,
    field_id: String,
    value: Option<String>,
) -> Result<FieldValue, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();

    let existing_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM entity_field_values WHERE page_id = ? AND field_id = ?",
    )
    .bind(&page_id).bind(&field_id)
    .fetch_optional(&pool).await.map_err(|e| e.to_string())?;
    let was_insert = existing_id.is_none();
    let final_id = existing_id.unwrap_or(id);

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let before = if !was_insert && sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.entity_field_values, &final_id).await.map_err(|e| e.to_string())?)
    } else { None };

    sqlx::query(
        "INSERT INTO entity_field_values (id, page_id, field_id, value)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(page_id, field_id) DO UPDATE SET value = excluded.value",
    )
    .bind(&final_id).bind(&page_id).bind(&field_id).bind(&value)
    .execute(&mut *tx).await.map_err(|e| e.to_string())?;

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    let kind = if was_insert { journal::OpKind::Insert } else { journal::OpKind::Update };
    emit_for_row(&mut *tx, &TABLES.entity_field_values, &final_id, kind, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(FieldValue { id: final_id, page_id, field_id, value })
}

#[tauri::command]
pub async fn delete_field_value(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    page_id: String,
    field_id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Look up id before deleting so we can capture before-state for the op.
    let existing_id: Option<String> = sqlx::query_scalar(
        "SELECT id FROM entity_field_values WHERE page_id = ? AND field_id = ?",
    )
    .bind(&page_id).bind(&field_id)
    .fetch_optional(&pool).await.map_err(|e| e.to_string())?;

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;

    let (before, id_for_op) = match (sync_session.is_some(), existing_id.as_ref()) {
        (true, Some(id)) => (
            Some(load_row_via_schema(&mut *tx, &TABLES.entity_field_values, id).await.map_err(|e| e.to_string())?),
            Some(id.clone()),
        ),
        _ => (None, None),
    };

    sqlx::query("DELETE FROM entity_field_values WHERE page_id = ? AND field_id = ?")
        .bind(&page_id).bind(&field_id).execute(&mut *tx).await.map_err(|e| e.to_string())?;

    if let (Some(id), Some(_)) = (id_for_op, before.as_ref()) {
        let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
        emit_for_row(&mut *tx, &TABLES.entity_field_values, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
            .await.map_err(|e| e.to_string())?;
    }
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();
    Ok(())
}

#[tauri::command]
pub async fn query_pages_by_field(
    managed: State<'_, ManagedDb>,
    field_id: String,
    value: String,
) -> Result<Vec<PageByFieldResult>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>)>(
        "SELECT p.id, p.title, p.icon, p.entity_type_id
         FROM pages p
         JOIN entity_field_values efv ON efv.page_id = p.id
         WHERE efv.field_id = ? AND efv.value = ?
         ORDER BY p.title",
    )
    .bind(&field_id)
    .bind(&value)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| PageByFieldResult {
            id: row.0,
            title: row.1,
            icon: row.2,
            entity_type_id: row.3,
        })
        .collect())
}
