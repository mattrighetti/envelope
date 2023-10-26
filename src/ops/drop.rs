use std::io::{Error, ErrorKind, Result};

use sqlx::SqlitePool;

pub async fn drop(db: &SqlitePool, env: &str) -> Result<()> {
    sqlx::query("DELETE FROM environments WHERE env = ?")
        .bind(env)
        .execute(db)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::test_db;

    #[tokio::test]
    async fn test_drop() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'A', 'X'),
            ('dev', 'B', 'Y'),
            ('dev', 'C', 'Z'),
            ('loc', 'B', 'Y'),
            ('test', 'C', 'Z'),
            ('test', 'D', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'dev'")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(3, rows.len());

        let res = drop(&pool, "dev").await;
        assert!(res.is_ok());

        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'dev'")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert!(rows.is_empty());
        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'loc'")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(1, rows.len());
        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'test'")
            .fetch_all(&pool)
            .await
            .unwrap();
        assert_eq!(2, rows.len());
    }
}
