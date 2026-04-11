use sqlx::SqlitePool;

pub async fn run(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await?;

    let migrations: Vec<(&str, &str)> = vec![
        ("001_wiki_engine", include_str!("../../migrations/001_wiki_engine.sql")),
        ("002_entity_types", include_str!("../../migrations/002_entity_types.sql")),
        ("003_tome_metadata", include_str!("../../migrations/003_tome_metadata.sql")),
        ("004_images", include_str!("../../migrations/004_images.sql")),
    ];

    for (name, sql) in migrations {
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)",
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        if !applied {
            log::info!("Running migration: {}", name);

            let statements = split_sql_statements(sql);
            log::info!("Found {} statements to execute", statements.len());

            for (i, statement) in statements.iter().enumerate() {
                log::info!("Executing statement {}/{}: {}...", i + 1, statements.len(), &statement[..statement.len().min(60)]);
                match sqlx::query(statement).execute(pool).await {
                    Ok(_) => log::info!("Statement {}/{} OK", i + 1, statements.len()),
                    Err(e) => {
                        log::error!("Statement {}/{} FAILED: {}", i + 1, statements.len(), e);
                        log::error!("Full statement: {}", statement);
                        return Err(Box::new(e));
                    }
                }
            }

            sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                .bind(name)
                .execute(pool)
                .await?;

            log::info!("Migration applied: {}", name);
        }
    }

    Ok(())
}

fn split_sql_statements(sql: &str) -> Vec<String> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut paren_depth: i32 = 0;

    for line in sql.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("--") {
            continue;
        }

        for ch in trimmed.chars() {
            if ch == '(' { paren_depth += 1; }
            if ch == ')' { paren_depth -= 1; }
        }

        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(trimmed);

        if trimmed.ends_with(';') && paren_depth == 0 {
            let stmt = current.trim_end_matches(';').trim().to_string();
            if !stmt.is_empty() {
                statements.push(stmt);
            }
            current.clear();
        }
    }

    let remaining = current.trim().trim_end_matches(';').trim().to_string();
    if !remaining.is_empty() {
        statements.push(remaining);
    }

    statements
}
