use sqlx::SqlitePool;
use std::io::{self, BufRead, Write};
use std::io::{Error, ErrorKind};

/// Adds a single key-value element to the database
///
/// If the value of v is None, an empty string is inserted
pub async fn add_var(db: &SqlitePool, env: &str, k: &str, v: &str) -> io::Result<()> {
    if k.starts_with('#') {
        return Err(Error::new(ErrorKind::Other, "key name cannot start with #"));
    }

    sqlx::query("INSERT INTO environments(env,key,value) VALUES (?, upper(?), ?);")
        .bind(env)
        .bind(k)
        .bind(v)
        .execute(db)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

pub async fn import<R: BufRead>(reader: R, pool: &SqlitePool, env: &str) -> io::Result<()> {
    for line in reader.lines() {
        if line.is_err() {
            continue;
        }

        if line.as_ref().unwrap().starts_with('#') {
            writeln!(io::stdout(), "skipping {}", line.unwrap())?;
            continue;
        }

        if let Some((k, v)) = line.unwrap().split_once('=') {
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
