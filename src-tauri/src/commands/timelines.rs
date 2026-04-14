use crate::db::{self, ManagedDb};
use crate::sync::journal::{self, active_sync_session, emit_for_row, load_row_via_schema};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Timeline {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TimelineEvent {
    pub id: String,
    pub timeline_id: String,
    pub title: String,
    pub description: Option<String>,
    pub date: String,
    pub end_date: Option<String>,
    pub page_id: Option<String>,
    pub color: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_timeline(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    name: String,
    description: Option<String>,
) -> Result<Timeline, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO timelines (id, name, description, sort_order, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)")
        .bind(&id).bind(&name).bind(&description).bind(&now).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.timelines, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(Timeline { id, name, description, sort_order: 0, created_at: now.clone(), updated_at: now })
}

#[tauri::command]
pub async fn list_timelines(managed: State<'_, ManagedDb>) -> Result<Vec<Timeline>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, i64, String, String)>(
        "SELECT id, name, description, sort_order, created_at, updated_at FROM timelines ORDER BY sort_order, name",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| Timeline {
        id: r.0, name: r.1, description: r.2, sort_order: r.3, created_at: r.4, updated_at: r.5,
    }).collect())
}

#[tauri::command]
pub async fn delete_timeline(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.timelines, &id).await.map_err(|e| e.to_string())?)
    } else { None };
    sqlx::query("DELETE FROM timelines WHERE id = ?").bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.timelines, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_timeline_event(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    timeline_id: String,
    title: String,
    date: String,
    description: Option<String>,
    end_date: Option<String>,
    page_id: Option<String>,
    color: Option<String>,
) -> Result<TimelineEvent, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO timeline_events (id, timeline_id, title, description, date, end_date, page_id, color, sort_order, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, 0, ?)")
        .bind(&id).bind(&timeline_id).bind(&title).bind(&description).bind(&date).bind(&end_date).bind(&page_id).bind(&color).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.timeline_events, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(TimelineEvent { id, timeline_id, title, description, date, end_date, page_id, color, sort_order: 0, created_at: now })
}

#[tauri::command]
pub async fn update_timeline_event(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    id: String,
    title: Option<String>,
    date: Option<String>,
    description: Option<String>,
    end_date: Option<String>,
    page_id: Option<String>,
    color: Option<String>,
) -> Result<TimelineEvent, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.timeline_events, &id).await.map_err(|e| e.to_string())?)
    } else { None };

    if let Some(ref v) = title { sqlx::query("UPDATE timeline_events SET title = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = date { sqlx::query("UPDATE timeline_events SET date = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = description { sqlx::query("UPDATE timeline_events SET description = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = end_date { sqlx::query("UPDATE timeline_events SET end_date = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = page_id { sqlx::query("UPDATE timeline_events SET page_id = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = color { sqlx::query("UPDATE timeline_events SET color = ? WHERE id = ?").bind(v).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.timeline_events, &id, journal::OpKind::Update, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;

    let row = sqlx::query_as::<_, (String, String, String, Option<String>, String, Option<String>, Option<String>, Option<String>, i64, String)>(
        "SELECT id, timeline_id, title, description, date, end_date, page_id, color, sort_order, created_at FROM timeline_events WHERE id = ?",
    ).bind(&id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;

    Ok(TimelineEvent { id: row.0, timeline_id: row.1, title: row.2, description: row.3, date: row.4, end_date: row.5, page_id: row.6, color: row.7, sort_order: row.8, created_at: row.9 })
}

#[tauri::command]
pub async fn delete_timeline_event(
    managed: State<'_, ManagedDb>,
    _session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.timeline_events, &id).await.map_err(|e| e.to_string())?)
    } else { None };
    sqlx::query("DELETE FROM timeline_events WHERE id = ?").bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.timeline_events, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_timeline_events(managed: State<'_, ManagedDb>, timeline_id: String) -> Result<Vec<TimelineEvent>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, String, Option<String>, String, Option<String>, Option<String>, Option<String>, i64, String)>(
        "SELECT id, timeline_id, title, description, date, end_date, page_id, color, sort_order, created_at FROM timeline_events WHERE timeline_id = ? ORDER BY date, sort_order",
    ).bind(&timeline_id).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| TimelineEvent {
        id: r.0, timeline_id: r.1, title: r.2, description: r.3, date: r.4, end_date: r.5, page_id: r.6, color: r.7, sort_order: r.8, created_at: r.9,
    }).collect())
}
