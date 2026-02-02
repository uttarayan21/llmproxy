use anyhow::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;
use tracing::info;

pub async fn create_pool(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    info!("Database connection pool created");
    Ok(pool)
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<()> {
    info!("Running database migrations");

    let migration_sql = include_str!("../migrations/001_init.sql");

    sqlx::query(migration_sql).execute(pool).await?;

    info!("Database migrations completed");
    Ok(())
}
