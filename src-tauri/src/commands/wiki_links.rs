use crate::db::DbPool;
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct WikiLinkInput {
    pub target_page_id: String,
    pub link_text: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BacklinkResult {
    pub page_id: String,
    pub title: String,
    pub entity_type_id: Option<String>,
}

#[tauri::command]
pub async fn save_wiki_links(
    pool: State<'_, DbPool>,
    source_page_id: String,
    links: Vec<WikiLinkInput>,
) -> Result<(), String> {
    // Delete existing links from this page
    sqlx::query("DELETE FROM wiki_links WHERE source_page_id = ?")
        .bind(&source_page_id)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;

    // Insert new links
    for link in links {
        sqlx::query(
            "INSERT OR IGNORE INTO wiki_links (source_page_id, target_page_id, link_text) VALUES (?, ?, ?)",
        )
        .bind(&source_page_id)
        .bind(&link.target_page_id)
        .bind(&link.link_text)
        .execute(pool.inner())
        .await
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn get_backlinks(
    pool: State<'_, DbPool>,
    page_id: String,
) -> Result<Vec<BacklinkResult>, String> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
        "SELECT p.id, p.title, p.entity_type_id
         FROM wiki_links wl
         JOIN pages p ON p.id = wl.source_page_id
         WHERE wl.target_page_id = ?
         ORDER BY p.title",
    )
    .bind(&page_id)
    .fetch_all(pool.inner())
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| BacklinkResult {
            page_id: row.0,
            title: row.1,
            entity_type_id: row.2,
        })
        .collect())
}
