use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Board {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardCard {
    pub id: String,
    pub board_id: String,
    pub page_id: Option<String>,
    pub content: Option<String>,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub color: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoardConnector {
    pub id: String,
    pub board_id: String,
    pub source_card_id: String,
    pub target_card_id: String,
    pub label: Option<String>,
    pub color: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_board(managed: State<'_, ManagedDb>, name: String) -> Result<Board, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO boards (id, name, sort_order, created_at, updated_at) VALUES (?, ?, 0, ?, ?)")
        .bind(&id).bind(&name).bind(&now).bind(&now)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(Board { id, name, sort_order: 0, created_at: now.clone(), updated_at: now })
}

#[tauri::command]
pub async fn list_boards(managed: State<'_, ManagedDb>) -> Result<Vec<Board>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, i64, String, String)>(
        "SELECT id, name, sort_order, created_at, updated_at FROM boards ORDER BY sort_order, name",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| Board { id: r.0, name: r.1, sort_order: r.2, created_at: r.3, updated_at: r.4 }).collect())
}

#[tauri::command]
pub async fn delete_board(managed: State<'_, ManagedDb>, id: String) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("DELETE FROM boards WHERE id = ?").bind(&id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn create_card(
    managed: State<'_, ManagedDb>, board_id: String, x: f64, y: f64,
    content: Option<String>, page_id: Option<String>, color: Option<String>,
) -> Result<BoardCard, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO board_cards (id, board_id, page_id, content, x, y, width, height, color, created_at) VALUES (?, ?, ?, ?, ?, ?, 200, 120, ?, ?)")
        .bind(&id).bind(&board_id).bind(&page_id).bind(&content).bind(x).bind(y).bind(&color).bind(&now)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(BoardCard { id, board_id, page_id, content, x, y, width: 200.0, height: 120.0, color, created_at: now })
}

#[tauri::command]
pub async fn update_card(
    managed: State<'_, ManagedDb>, id: String,
    x: Option<f64>, y: Option<f64>, content: Option<String>,
    page_id: Option<String>, color: Option<String>,
    width: Option<f64>, height: Option<f64>,
) -> Result<BoardCard, String> {
    let pool = db::get_pool(managed.inner()).await?;
    if let Some(v) = x { sqlx::query("UPDATE board_cards SET x = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(v) = y { sqlx::query("UPDATE board_cards SET y = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = content { sqlx::query("UPDATE board_cards SET content = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = page_id { sqlx::query("UPDATE board_cards SET page_id = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(ref v) = color { sqlx::query("UPDATE board_cards SET color = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(v) = width { sqlx::query("UPDATE board_cards SET width = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }
    if let Some(v) = height { sqlx::query("UPDATE board_cards SET height = ? WHERE id = ?").bind(v).bind(&id).execute(&pool).await.map_err(|e| e.to_string())?; }

    let row = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, f64, f64, f64, f64, Option<String>, String)>(
        "SELECT id, board_id, page_id, content, x, y, width, height, color, created_at FROM board_cards WHERE id = ?",
    ).bind(&id).fetch_one(&pool).await.map_err(|e| e.to_string())?;
    Ok(BoardCard { id: row.0, board_id: row.1, page_id: row.2, content: row.3, x: row.4, y: row.5, width: row.6, height: row.7, color: row.8, created_at: row.9 })
}

#[tauri::command]
pub async fn delete_card(managed: State<'_, ManagedDb>, id: String) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("DELETE FROM board_cards WHERE id = ?").bind(&id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_board_cards(managed: State<'_, ManagedDb>, board_id: String) -> Result<Vec<BoardCard>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, f64, f64, f64, f64, Option<String>, String)>(
        "SELECT id, board_id, page_id, content, x, y, width, height, color, created_at FROM board_cards WHERE board_id = ?",
    ).bind(&board_id).fetch_all(&pool).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| BoardCard { id: r.0, board_id: r.1, page_id: r.2, content: r.3, x: r.4, y: r.5, width: r.6, height: r.7, color: r.8, created_at: r.9 }).collect())
}

#[tauri::command]
pub async fn create_connector(
    managed: State<'_, ManagedDb>, board_id: String,
    source_card_id: String, target_card_id: String,
    label: Option<String>, color: Option<String>,
) -> Result<BoardConnector, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query("INSERT INTO board_connectors (id, board_id, source_card_id, target_card_id, label, color, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&board_id).bind(&source_card_id).bind(&target_card_id).bind(&label).bind(&color).bind(&now)
        .execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(BoardConnector { id, board_id, source_card_id, target_card_id, label, color, created_at: now })
}

#[tauri::command]
pub async fn delete_connector(managed: State<'_, ManagedDb>, id: String) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    sqlx::query("DELETE FROM board_connectors WHERE id = ?").bind(&id).execute(&pool).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_board_connectors(managed: State<'_, ManagedDb>, board_id: String) -> Result<Vec<BoardConnector>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    let rows = sqlx::query_as::<_, (String, String, String, String, Option<String>, Option<String>, String)>(
        "SELECT id, board_id, source_card_id, target_card_id, label, color, created_at FROM board_connectors WHERE board_id = ?",
    ).bind(&board_id).fetch_all(&pool).await.map_err(|e| e.to_string())?;
    Ok(rows.into_iter().map(|r| BoardConnector { id: r.0, board_id: r.1, source_card_id: r.2, target_card_id: r.3, label: r.4, color: r.5, created_at: r.6 }).collect())
}
