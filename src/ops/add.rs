use sqlx::SqlitePool;
use super::read_lines;

use std::io;
use std::io::{Error, ErrorKind};

/// Adds a single key-value element to the database
///
/// If the value of v is None, an empty string is inserted
pub async fn add_var(db: &SqlitePool, env: &str, k: &str, v: &str) -> io::Result<()> {
    sqlx::query("INSERT INTO environments(env,key,value) VALUES (?, upper(?), ?);")
        .bind(env)
        .bind(k)
        .bind(v)
        .execute(db)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

/// Imports every key-value from a file at the given path
pub async fn import_from(pool: &SqlitePool, env: &str, path: &str) -> io::Result<()> {
    let buf = read_lines(path)?;
    for line in buf {
        if line.is_err() {
            continue
        }

        if let Some((k, v)) = line.unwrap().split_once("=") {
            sqlx::query("INSERT INTO environments(env,key,value) VALUES (?, upper(?), ?);")
                .bind(env)
                .bind(k)
                .bind(v)
                .execute(pool)
                .await
                .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;
        }
    }

    Ok(())
}
