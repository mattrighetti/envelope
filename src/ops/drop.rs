use std::io::Result;

use crate::db::EnvelopeDb;

pub async fn drop(db: &EnvelopeDb, env: &str) -> Result<()> {
    db.drop_env(env).await
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::test_db;

    #[tokio::test]
    async fn test_drop() {
        let db = test_db().await;
        let pool = db.get_pool();

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
        .execute(pool)
        .await
        .unwrap();

        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'dev'")
            .fetch_all(pool)
            .await
            .unwrap();
        assert_eq!(3, rows.len());

        let res = drop(&db, "dev").await;
        assert!(res.is_ok());

        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'dev'")
            .fetch_all(pool)
            .await
            .unwrap();
        assert!(rows.is_empty());
        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'loc'")
            .fetch_all(pool)
            .await
            .unwrap();
        assert_eq!(1, rows.len());
        let rows = sqlx::query("SELECT * FROM environments WHERE env = 'test'")
            .fetch_all(pool)
            .await
            .unwrap();
        assert_eq!(2, rows.len());
    }
}
