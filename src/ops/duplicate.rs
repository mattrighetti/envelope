use sqlx::SqlitePool;
use std::io;
use std::io::{Error, ErrorKind};

pub async fn duplicate(pool: &SqlitePool, source: &str, target: &str) -> io::Result<()> {
    sqlx::query(
        r"INSERT INTO environments(env,key,value)
        SELECT ?2, key, value
        FROM environments WHERE env = ?1 AND value NOT NULL
        GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key;",
    )
    .bind(source)
    .bind(target)
    .execute(pool)
    .await
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}
