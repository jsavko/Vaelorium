use crate::db::{self, ManagedDb};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub pages_imported: usize,
    pub errors: Vec<String>,
}

#[tauri::command]
pub async fn import_markdown_folder(
    managed: State<'_, ManagedDb>,
    path: String,
) -> Result<ImportResult, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let mut pages_imported = 0;
    let mut errors = Vec::new();

    let entries = std::fs::read_dir(&path).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => { errors.push(e.to_string()); continue; }
        };

        let file_path = entry.path();
        if !file_path.extension().map(|e| e == "md").unwrap_or(false) {
            continue;
        }

        let content = match std::fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => { errors.push(format!("{}: {}", file_path.display(), e)); continue; }
        };

        // Parse frontmatter
        let (title, body) = parse_markdown(&content, &file_path);

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();

        // Create page
        match sqlx::query(
            "INSERT INTO pages (id, title, sort_order, visibility, created_at, updated_at) VALUES (?, ?, 0, 'private', ?, ?)",
        )
        .bind(&id).bind(&title).bind(&now).bind(&now)
        .execute(&pool).await {
            Ok(_) => {
                // Create empty Yjs content
                let empty: Vec<u8> = vec![];
                sqlx::query("INSERT INTO page_content (page_id, yjs_state, yjs_version) VALUES (?, ?, 0)")
                    .bind(&id).bind(&empty)
                    .execute(&pool).await.ok();

                // Store the body text in FTS for searchability
                if !body.is_empty() {
                    sqlx::query("INSERT INTO pages_fts_content (page_id, title, text_content) VALUES (?, ?, ?)")
                        .bind(&id).bind(&title).bind(&body)
                        .execute(&pool).await.ok();
                }

                pages_imported += 1;
            }
            Err(e) => errors.push(format!("{}: {}", title, e)),
        }
    }

    Ok(ImportResult { pages_imported, errors })
}

#[tauri::command]
pub async fn import_json(
    managed: State<'_, ManagedDb>,
    json: String,
) -> Result<ImportResult, String> {
    let pool = db::get_pool(managed.inner()).await?;

    let export: serde_json::Value = serde_json::from_str(&json)
        .map_err(|e| format!("Invalid JSON: {}", e))?;

    let mut pages_imported = 0;
    let mut errors = Vec::new();

    // Import pages
    if let Some(pages) = export.get("pages").and_then(|v| v.as_array()) {
        for page in pages {
            let id = page.get("id").and_then(|v| v.as_str()).unwrap_or_default();
            let title = page.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled");
            let now = chrono::Utc::now().to_rfc3339();

            let new_id = uuid::Uuid::new_v4().to_string();

            match sqlx::query(
                "INSERT INTO pages (id, title, icon, entity_type_id, parent_id, sort_order, visibility, created_at, updated_at) VALUES (?, ?, ?, ?, ?, 0, 'private', ?, ?)",
            )
            .bind(&new_id)
            .bind(title)
            .bind(page.get("icon").and_then(|v| v.as_str()))
            .bind(page.get("entity_type_id").and_then(|v| v.as_str()))
            .bind::<Option<&str>>(None) // Don't preserve parent hierarchy on import
            .bind(&now).bind(&now)
            .execute(&pool).await {
                Ok(_) => {
                    // Create content
                    let empty: Vec<u8> = vec![];
                    sqlx::query("INSERT INTO page_content (page_id, yjs_state, yjs_version) VALUES (?, ?, 0)")
                        .bind(&new_id).bind(&empty)
                        .execute(&pool).await.ok();
                    pages_imported += 1;
                }
                Err(e) => errors.push(format!("Page '{}': {}", title, e)),
            }
        }
    }

    Ok(ImportResult { pages_imported, errors })
}

fn parse_markdown(content: &str, path: &std::path::Path) -> (String, String) {
    let filename = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled")
        .to_string();

    // Check for YAML frontmatter
    if content.starts_with("---") {
        if let Some(end) = content[3..].find("---") {
            let frontmatter = &content[3..3 + end];
            let body = content[3 + end + 3..].trim().to_string();

            // Extract title from frontmatter
            for line in frontmatter.lines() {
                let line = line.trim();
                if line.starts_with("title:") {
                    let title = line[6..].trim().trim_matches('"').to_string();
                    if !title.is_empty() {
                        return (title, body);
                    }
                }
            }
            return (filename, body);
        }
    }

    (filename, content.to_string())
}
