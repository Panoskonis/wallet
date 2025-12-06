use sqlx::{PgPool, Pool, Postgres};
use std::fs;
use std::path::Path;

pub type DbPool = Pool<Postgres>;

pub async fn create_pool(database_url: &str) -> anyhow::Result<DbPool> {
    // Create a connection pool with configuration
    let pool = PgPool::connect_with(
        // Parse the database URL into connection options
        database_url.parse()?,
    )
    .await?;

    // Verify the connection by running a simple query
    // This ensures the database is accessible before proceeding
    sqlx::query("SELECT 1").execute(&pool).await?;

    Ok(pool)
}

pub async fn health_check(pool: &DbPool) -> anyhow::Result<()> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}

pub async fn run_migrations(pool: &DbPool) -> anyhow::Result<()> {
    // Create migrations tracking table if it doesn't exist
    // This table keeps track of which migrations have been applied
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS _sqlx_migrations (
            version BIGINT PRIMARY KEY,
            description TEXT NOT NULL,
            installed_on TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            success BOOLEAN NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    // Read migrations directory
    let migrations_dir = Path::new("migrations");
    if !migrations_dir.exists() {
        println!("⚠️  No migrations directory found, skipping migrations");
        return Ok(());
    }

    // Get all SQL files and sort them alphabetically
    // Migration files should be named with timestamps for ordering (e.g., 20240101000001_name.sql)
    let mut migration_files: Vec<_> = fs::read_dir(migrations_dir)?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "sql" {
                Some(path)
            } else {
                None
            }
        })
        .collect();
    migration_files.sort();

    println!("📦 Found {} migration file(s)", migration_files.len());

    // Apply each migration
    for migration_file in migration_files {
        let filename = migration_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Extract version from filename (assumes format: YYYYMMDDHHMMSS_description.sql)
        let version: i64 = filename
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse()
            .unwrap_or(0);

        // Check if migration has already been applied
        let already_applied: Option<(bool,)> =
            sqlx::query_as("SELECT success FROM _sqlx_migrations WHERE version = $1")
                .bind(version)
                .fetch_optional(pool)
                .await?;

        if let Some((true,)) = already_applied {
            println!("⏭️  Skipping already applied migration: {}", filename);
            continue;
        }

        // Read migration SQL
        let sql = fs::read_to_string(&migration_file)?;
        println!("🔄 Applying migration: {}", filename);

        // Execute migration within a transaction
        // If migration fails, transaction is rolled back
        let mut tx = pool.begin().await?;

        // Split SQL into individual statements
        // PostgreSQL requires each statement to be executed separately
        // We split by semicolon and filter out empty/whitespace-only statements
        // Note: This simple approach works for DDL statements (CREATE, ALTER, etc.)
        // which typically don't have semicolons inside string literals
        let statements: Vec<String> = sql
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| {
                // Filter out empty strings and pure comment blocks
                let trimmed = s.trim();
                !trimmed.is_empty()
                    && !trimmed
                        .lines()
                        .all(|line| line.trim().starts_with("--") || line.trim().is_empty())
            })
            .collect();

        // Execute each statement individually
        for (idx, statement) in statements.iter().enumerate() {
            // Skip if statement is empty after processing
            let cleaned = statement.trim();
            if cleaned.is_empty() {
                continue;
            }

            match sqlx::query(cleaned).execute(&mut *tx).await {
                Ok(_) => {
                    // Statement executed successfully
                }
                Err(e) => {
                    // Record failed migration
                    sqlx::query(
                        "INSERT INTO _sqlx_migrations (version, description, success) 
                         VALUES ($1, $2, false)
                         ON CONFLICT (version) DO UPDATE SET success = false",
                    )
                    .bind(version)
                    .bind(&filename)
                    .execute(&mut *tx)
                    .await
                    .ok();

                    tx.rollback().await?;
                    return Err(anyhow::anyhow!(
                        "Migration {} failed at statement {}: {}",
                        filename,
                        idx + 1,
                        e
                    ));
                }
            }
        }

        // Record successful migration
        sqlx::query(
            "INSERT INTO _sqlx_migrations (version, description, success) 
             VALUES ($1, $2, true)
             ON CONFLICT (version) DO UPDATE SET success = true",
        )
        .bind(version)
        .bind(&filename)
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        println!("✅ Successfully applied migration: {}", filename);
    }

    println!("✅ All migrations applied successfully");
    Ok(())
}
