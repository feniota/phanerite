//! Compile-time migration discovery and the future runtime migration entry point.
//!
//! `build.rs` validates the files in `migrations` and embeds their contents
//! into the binary. Runtime application is intentionally not wired up yet because
//! the database integration is still being built.

pub mod migration_scripts {
    include!(concat!(env!("OUT_DIR"), "/migration_scripts.rs"));
}

pub async fn apply_pending(db: &super::Database) -> turso::Result<()> {
    let connection = db.inner.connect()?;
    connection
        .execute_batch(
            "CREATE TABLE IF NOT EXISTS __phanerite_migrations (
                 id INTEGER PRIMARY KEY,
                 slug TEXT NOT NULL,
                 applied_at TEXT NOT NULL DEFAULT (datetime('now'))
             );",
        )
        .await?;

    for migration in migration_scripts::ALL {
        // Query __phanerite_migrations here, skip applied IDs, then execute
        // each migration and record it in one transaction.
        let mut existing = connection
            .query(
                "SELECT id, slug FROM __phanerite_migrations WHERE id = ?1",
                (migration.id,),
            )
            .await?;
        if existing.next().await?.is_some() {
            tracing::info!(
                "Migration {} (id: {}) is applied, skipping",
                migration.slug,
                migration.id
            );
            continue;
        }

        connection.execute_batch(migration.sql).await?;
        connection
            .execute(
                "INSERT INTO __phanerite_migrations (id, slug) VALUES (?1, ?2)",
                (migration.id, migration.slug),
            )
            .await?;
        tracing::info!(
            "Successfully applied migration {} (id: {})",
            migration.slug,
            migration.id
        );
    }
    Ok(())
}
