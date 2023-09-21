use std::env;
use sqlx::SqlitePool;

pub(crate) type EnvelopeResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnvironmentRow {
    pub env: String,
    pub key: String,
    pub value: String,
    pub created_at: i32
}

/// Checks if an `.envelope` file is present in the current directory,
/// if it is nothing is done and an error in returned, otherwise a new envelope
/// database will get created
pub async fn init() -> EnvelopeResult<SqlitePool> {
    let envelope_fs = env::current_dir()?.join(".envelope");

    let db_path = envelope_fs
        .into_os_string()
        .into_string()
        .unwrap();

    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=rwc", db_path))
        .await
        .map_err(|err| format!("{}\nfile: {}", err, db_path))?;

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS environments(
        env VARCHAR(50) NOT NULL,
        key TEXT NOT NULL,
        value TEXT,
        created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
        PRIMARY KEY(env,key,created_at));"#
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
