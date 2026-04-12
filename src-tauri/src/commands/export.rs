use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct TomeExport {
    pub version: String,
    pub pages: Vec<PageExport>,
    pub entity_types: Vec<serde_json::Value>,
    pub entity_type_fields: Vec<serde_json::Value>,
    pub entity_field_values: Vec<serde_json::Value>,
    pub relations: Vec<serde_json::Value>,
    pub relation_types: Vec<serde_json::Value>,
    pub maps: Vec<serde_json::Value>,
    pub map_pins: Vec<serde_json::Value>,
    pub timelines: Vec<serde_json::Value>,
    pub timeline_events: Vec<serde_json::Value>,
    pub boards: Vec<serde_json::Value>,
    pub board_cards: Vec<serde_json::Value>,
    pub board_connectors: Vec<serde_json::Value>,
    pub tags: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PageExport {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub entity_type_id: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub visibility: String,
    pub created_at: String,
    pub updated_at: String,
    pub content_base64: Option<String>,
}

async fn query_all_json(pool: &sqlx::SqlitePool, table: &str) -> Result<Vec<serde_json::Value>, String> {
    // Use SQLite's json functions to serialize rows
    let query = format!(
        "SELECT json_group_array(json(row_json)) FROM (SELECT json_object(*) as row_json FROM {})",
        table
    );
    let result: Option<String> = sqlx::query_scalar(&query)
        .fetch_optional(pool)
        .await
        .map_err(|e| e.to_string())?;

    match result {
        Some(json_str) => {
            serde_json::from_str(&json_str).map_err(|e| e.to_string())
        }
        None => Ok(Vec::new()),
    }
}

#[tauri::command]
pub async fn export_tome_json(managed: State<'_, ManagedDb>) -> Result<String, String> {
    let pool = db::get_pool(managed.inner()).await?;

    // Export pages with content
    let page_rows = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, Option<String>, i64, String, String, String)>(
        "SELECT id, title, icon, entity_type_id, parent_id, sort_order, visibility, created_at, updated_at FROM pages ORDER BY sort_order",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    let mut pages = Vec::new();
    for row in page_rows {
        let content: Option<Vec<u8>> = sqlx::query_scalar(
            "SELECT yjs_state FROM page_content WHERE page_id = ?",
        ).bind(&row.0).fetch_optional(&pool).await.map_err(|e| e.to_string())?;

        let content_base64 = content
            .filter(|c| !c.is_empty())
            .map(|c| base64_encode(&c));

        pages.push(PageExport {
            id: row.0, title: row.1, icon: row.2, entity_type_id: row.3,
            parent_id: row.4, sort_order: row.5, visibility: row.6,
            created_at: row.7, updated_at: row.8, content_base64,
        });
    }

    let export = TomeExport {
        version: "1.0".to_string(),
        pages,
        entity_types: query_all_json(&pool, "entity_types").await?,
        entity_type_fields: query_all_json(&pool, "entity_type_fields").await?,
        entity_field_values: query_all_json(&pool, "entity_field_values").await?,
        relations: query_all_json(&pool, "relations").await?,
        relation_types: query_all_json(&pool, "relation_types").await?,
        maps: query_all_json(&pool, "maps").await?,
        map_pins: query_all_json(&pool, "map_pins").await?,
        timelines: query_all_json(&pool, "timelines").await?,
        timeline_events: query_all_json(&pool, "timeline_events").await?,
        boards: query_all_json(&pool, "boards").await?,
        board_cards: query_all_json(&pool, "board_cards").await?,
        board_connectors: query_all_json(&pool, "board_connectors").await?,
        tags: query_all_json(&pool, "tags").await?,
    };

    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_tome_markdown(
    managed: State<'_, ManagedDb>,
    path: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;

    std::fs::create_dir_all(&path).map_err(|e| e.to_string())?;

    let pages = sqlx::query_as::<_, (String, String, Option<String>, Option<String>, String, String)>(
        "SELECT id, title, icon, entity_type_id, created_at, updated_at FROM pages ORDER BY sort_order",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;

    // Get entity type names for frontmatter
    let type_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM entity_types",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;
    let type_map: HashMap<String, String> = type_rows.into_iter().collect();

    // Get field values for frontmatter
    let field_rows = sqlx::query_as::<_, (String, String)>(
        "SELECT id, name FROM entity_type_fields",
    ).fetch_all(&pool).await.map_err(|e| e.to_string())?;
    let field_map: HashMap<String, String> = field_rows.into_iter().collect();

    for page in &pages {
        let safe_title = page.1.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
        let filename = format!("{}.md", safe_title);
        let filepath = std::path::Path::new(&path).join(&filename);

        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("title: \"{}\"\n", page.1));
        if let Some(ref icon) = page.2 {
            md.push_str(&format!("icon: \"{}\"\n", icon));
        }
        if let Some(ref type_id) = page.3 {
            if let Some(type_name) = type_map.get(type_id) {
                md.push_str(&format!("type: \"{}\"\n", type_name));
            }
        }
        md.push_str(&format!("created: \"{}\"\n", page.4));
        md.push_str(&format!("updated: \"{}\"\n", page.5));

        // Add field values
        let values = sqlx::query_as::<_, (String, Option<String>)>(
            "SELECT field_id, value FROM entity_field_values WHERE page_id = ?",
        ).bind(&page.0).fetch_all(&pool).await.map_err(|e| e.to_string())?;

        for (field_id, value) in &values {
            if let (Some(name), Some(val)) = (field_map.get(field_id), value) {
                md.push_str(&format!("{}: {}\n", name.to_lowercase().replace(' ', "_"), val));
            }
        }

        md.push_str("---\n\n");

        // Get page content as plain text (simplified — no Yjs parsing in Rust)
        let text: Option<String> = sqlx::query_scalar(
            "SELECT text_content FROM pages_fts_content WHERE page_id = ?",
        ).bind(&page.0).fetch_optional(&pool).await.map_err(|e| e.to_string())?;

        if let Some(content) = text {
            md.push_str(&content);
        }

        std::fs::write(&filepath, md).map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 { result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char); } else { result.push('='); }
        if chunk.len() > 2 { result.push(CHARS[(triple & 0x3F) as usize] as char); } else { result.push('='); }
    }
    result
}
