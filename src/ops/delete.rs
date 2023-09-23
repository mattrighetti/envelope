use std::io;
use std::io::{Error, ErrorKind};

use sqlx::SqlitePool;

/// Deletes every key found in an enviroment
pub async fn delete_env(pool: &SqlitePool, env: &str) -> io::Result<()> {
    sqlx::query("UPDATE environments SET value = NULL WHERE env = ?")
        .bind(env)
        .execute(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

/// Deletes a key for every environments
pub async fn delete_var_globally(pool: &SqlitePool, key: &str) -> io::Result<()> {
    sqlx::query("UPDATE environments SET value = NULL WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

/// Deletes a key in a specific env
pub async fn delete_var_in_env(pool: &SqlitePool, env: &str, key: &str) -> io::Result<()> {
    sqlx::query("UPDATE environments SET value = NULL WHERE env = ? AND key = ?")
        .bind(env)
        .bind(key)
        .execute(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}
