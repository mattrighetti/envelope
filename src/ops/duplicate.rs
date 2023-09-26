use sqlx::SqlitePool;
use std::io::{self, BufRead, Write};
use std::io::{Error, ErrorKind};

use super::read_lines;

pub async fn duplicate(pool: &SqlitePool, source: &str, target: &str) -> io::Result<()> {
    sqlx::query(
        r"INSERT INTO environments(env,key,value)
        SELECT ?2, key, value
        FROM environments WHERE env = ?1",
    )
    .bind(source)
    .bind(target)
    .execute(pool)
    .await
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}
