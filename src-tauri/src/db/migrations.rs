use sqlx::SqlitePool;

pub async fn run(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error>> {
    // Create migrations tracking table
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

            // Execute each statement separately (sqlx doesn't support multi-statement execute)
            for statement in sql.split(";\n") {
                let trimmed = statement.trim();
                if !trimmed.is_empty() && !trimmed.starts_with("--") {
                    sqlx::query(trimmed).execute(pool).await?;
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
