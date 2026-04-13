use crate::db::{self, ManagedDb};
use crate::sync::journal::{self, active_sync_session, emit_for_row, load_row_via_schema};
use crate::sync::registry::TABLES;
use crate::sync::SessionState;
use serde::{Deserialize, Serialize};
use tauri::State;
use ulid::Ulid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapInfo {
    pub id: String,
    pub title: String,
    pub image_id: Option<String>,
    pub parent_map_id: Option<String>,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MapPin {
    pub id: String,
    pub map_id: String,
    pub page_id: Option<String>,
    pub label: Option<String>,
    pub x: f64,
    pub y: f64,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_map(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    title: String,
    image_id: Option<String>,
) -> Result<MapInfo, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO maps (id, title, image_id, sort_order, created_at, updated_at) VALUES (?, ?, ?, 0, ?, ?)")
        .bind(&id).bind(&title).bind(&image_id).bind(&now).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.maps, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(MapInfo { id, title, image_id, parent_map_id: None, sort_order: 0, created_at: now.clone(), updated_at: now })
}

#[tauri::command]
pub async fn list_maps(managed: State<'_, ManagedDb>) -> Result<Vec<MapInfo>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, i64, String, String)>(
        "SELECT id, title, image_id, parent_map_id, sort_order, created_at, updated_at FROM maps ORDER BY sort_order, title",
    )
    .fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| MapInfo {
        id: r.0, title: r.1, image_id: r.2, parent_map_id: r.3, sort_order: r.4, created_at: r.5, updated_at: r.6,
    }).collect())
}

#[tauri::command]
pub async fn get_map(managed: State<'_, ManagedDb>, id: String) -> Result<MapInfo, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, i64, String, String)>(
        "SELECT id, title, image_id, parent_map_id, sort_order, created_at, updated_at FROM maps WHERE id = ?",
    )
    .bind(&id).fetch_optional(&pool).await.map_err(|e| e.to_string())?
    .ok_or_else(|| "Map not found".to_string())?;

    Ok(MapInfo { id: row.0, title: row.1, image_id: row.2, parent_map_id: row.3, sort_order: row.4, created_at: row.5, updated_at: row.6 })
}

#[tauri::command]
pub async fn delete_map(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.maps, &id).await.map_err(|e| e.to_string())?)
    } else { None };
    sqlx::query("DELETE FROM maps WHERE id = ?").bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.maps, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();
    Ok(())
}

#[tauri::command]
pub async fn create_pin(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    map_id: String,
    x: f64,
    y: f64,
    page_id: Option<String>,
    label: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Result<MapPin, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    sqlx::query("INSERT INTO map_pins (id, map_id, page_id, label, x, y, icon, color, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&map_id).bind(&page_id).bind(&label).bind(x).bind(y).bind(&icon).bind(&color).bind(&now)
        .execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.map_pins, &id, journal::OpKind::Insert, Ulid::new(), None, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(MapPin { id, map_id, page_id, label, x, y, icon, color, created_at: now })
}

#[tauri::command]
pub async fn update_pin(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    id: String,
    x: Option<f64>,
    y: Option<f64>,
    page_id: Option<String>,
    label: Option<String>,
    color: Option<String>,
) -> Result<MapPin, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.map_pins, &id).await.map_err(|e| e.to_string())?)
    } else { None };

    if let Some(px) = x { sqlx::query("UPDATE map_pins SET x = ? WHERE id = ?").bind(px).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(py) = y { sqlx::query("UPDATE map_pins SET y = ? WHERE id = ?").bind(py).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref pid) = page_id { sqlx::query("UPDATE map_pins SET page_id = ? WHERE id = ?").bind(pid).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref lbl) = label { sqlx::query("UPDATE map_pins SET label = ? WHERE id = ?").bind(lbl).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }
    if let Some(ref clr) = color { sqlx::query("UPDATE map_pins SET color = ? WHERE id = ?").bind(clr).bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?; }

    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.map_pins, &id, journal::OpKind::Update, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;

    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, f64, f64, Option<String>, Option<String>, String)>(
        "SELECT id, map_id, page_id, label, x, y, icon, color, created_at FROM map_pins WHERE id = ?",
    ).bind(&id).fetch_one(&mut *tx).await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();

    Ok(MapPin { id: row.0, map_id: row.1, page_id: row.2, label: row.3, x: row.4, y: row.5, icon: row.6, color: row.7, created_at: row.8 })
}

#[tauri::command]
pub async fn delete_pin(
    managed: State<'_, ManagedDb>,
    session: State<'_, SessionState>,
    id: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    let sync_session = active_sync_session(&pool).await.map_err(|e| e.to_string())?;
    let mut tx = pool.begin().await.map_err(|e| e.to_string())?;
    let before = if sync_session.is_some() {
        Some(load_row_via_schema(&mut *tx, &TABLES.map_pins, &id).await.map_err(|e| e.to_string())?)
    } else { None };
    sqlx::query("DELETE FROM map_pins WHERE id = ?").bind(&id).execute(&mut *tx).await.map_err(|e| e.to_string())?;
    let session_ref = sync_session.as_ref().map(|(t, d)| (t.as_str(), *d));
    emit_for_row(&mut *tx, &TABLES.map_pins, &id, journal::OpKind::Delete, Ulid::new(), before, session_ref)
        .await.map_err(|e| e.to_string())?;
    tx.commit().await.map_err(|e| e.to_string())?;
    session.nudge();
    Ok(())
}

#[tauri::command]
pub async fn get_map_pins(managed: State<'_, ManagedDb>, map_id: String) -> Result<Vec<MapPin>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, f64, f64, Option<String>, Option<String>, String)>(
        "SELECT id, map_id, page_id, label, x, y, icon, color, created_at FROM map_pins WHERE map_id = ?",
    ).bind(&map_id).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    Ok(rows.into_iter().map(|r| MapPin {
        id: r.0, map_id: r.1, page_id: r.2, label: r.3, x: r.4, y: r.5, icon: r.6, color: r.7, created_at: r.8,
    }).collect())
}
