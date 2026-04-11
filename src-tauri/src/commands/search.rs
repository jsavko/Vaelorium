use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub page_id: String,
    pub title: String,
    pub entity_type_id: Option<String>,
    pub snippet: Option<String>,
}

#[tauri::command]
pub async fn update_search_index(
    managed: State<'_, ManagedDb>,
    page_id: String,
    title: String,
    text_content: String,
) -> Result<(), String> {
    let pool = db::get_pool(managed.inner()).await?;
    // Upsert into backing table
    sqlx::query(
        "INSERT INTO pages_fts_content (page_id, title, text_content)
         VALUES (?, ?, ?)
         ON CONFLICT(page_id) DO UPDATE SET title = excluded.title, text_content = excluded.text_content",
    )
    .bind(&page_id)
    .bind(&title)
    .bind(&text_content)
    .execute(&pool)
    .await
    .map_err(|e| e.to_string())?;

    // Rebuild FTS index for this row
    let rowid: Option<i64> = sqlx::query_scalar(
        "SELECT rowid FROM pages_fts_content WHERE page_id = ?",
    )
    .bind(&page_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| e.to_string())?;

    if let Some(rid) = rowid {
        // Delete old FTS entry and reinsert
        sqlx::query("INSERT INTO pages_fts(pages_fts, rowid, title, text_content) VALUES('delete', ?, ?, ?)")
            .bind(rid)
            .bind(&title)
            .bind(&text_content)
            .execute(&pool)
            .await
            .ok(); // Ignore error if entry didn't exist

        sqlx::query("INSERT INTO pages_fts(rowid, title, text_content) VALUES(?, ?, ?)")
            .bind(rid)
            .bind(&title)
            .bind(&text_content)
            .execute(&pool)
            .await
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
pub async fn search_pages(
    managed: State<'_, ManagedDb>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let pool = db::get_pool(managed.inner()).await?;
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    // Add prefix matching
    let fts_query = format!("{}*", query.trim());

    let rows = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "SELECT fc.page_id, fc.title, p.entity_type_id, snippet(pages_fts, 1, '<b>', '</b>', '...', 32) as snippet
         FROM pages_fts
         JOIN pages_fts_content fc ON fc.rowid = pages_fts.rowid
         JOIN pages p ON p.id = fc.page_id
         WHERE pages_fts MATCH ?
         ORDER BY rank
         LIMIT 20",
    )
    .bind(&fts_query)
    .fetch_all(&pool)
    .await
    .map_err(|e| e.to_string())?;

    Ok(rows
        .into_iter()
        .map(|row| SearchResult {
            page_id: row.0,
            title: row.1,
            entity_type_id: row.2,
            snippet: Some(row.3),
        })
        .collect())
}
