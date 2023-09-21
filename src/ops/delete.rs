use std::io;
use std::io::{Error, ErrorKind};

use sqlx::SqlitePool;

pub async fn delete_env(pool: &SqlitePool, env: &str) -> io::Result<()> {
    sqlx::query("DELETE FROM environments WHERE env = ?")
        .bind(env)
        .execute(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}
