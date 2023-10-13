use std::io;
use std::io::{Error, ErrorKind};

use sqlx::{QueryBuilder, Sqlite, SqlitePool};

fn query_builder(overwrite: &bool) -> QueryBuilder<Sqlite> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r"INSERT INTO environments (env, key, value)
        SELECT ?2, key, value
        FROM environments
        WHERE env = ?1 ",
    );
    if !overwrite {
        query_builder.push(r" AND key NOT IN (SELECT key FROM environments WHERE env = ?2 GROUP BY key HAVING MAX(created_at)) ");
    }
    query_builder.push("GROUP BY key HAVING MAX(created_at);");

    query_builder
}

/// Syncs copies variables src_env and adds them to target_env
/// only if they are not already present
pub async fn sync(
    db: &SqlitePool,
    src_env: &str,
    target_env: &str,
    overwrite: bool,
) -> io::Result<()> {
    query_builder(&overwrite)
        .build()
        .bind(src_env)
        .bind(target_env)
        .execute(db)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::{test_db, EnvironmentRow};

    #[tokio::test]
    async fn test_empty_sync() {
        let pool = test_db().await;
        let res = sync(&pool, "dev", "prod", false).await;

        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_sync_1() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'A', 'X'),
            ('dev', 'B', 'Y'),
            ('dev', 'C', 'Z'),
            ('dev', 'D', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = sync(&pool, "dev", "prod", false).await;
        assert!(res.is_ok());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let expected_keys = ["A", "B", "C", "D"];
        let expected_values = ["X", "Y", "Z", "K"];
        assert_eq!(4, rows.len());
        for i in 0..4 {
            assert_eq!(expected_keys[i], rows[i].key);
            assert_eq!(expected_values[i], rows[i].value);
        }
    }

    #[tokio::test]
    async fn test_sync_2() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('prod', 'C', '3'),
            ('prod', 'D', '2'),
            ('prod', 'F', '1'),
            ('dev',  'A', 'X'),
            ('dev',  'B', 'Y'),
            ('dev',  'C', 'Z'),
            ('dev',  'D', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = sync(&pool, "dev", "prod", false).await;
        assert!(res.is_ok());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let expected_keys = ["A", "B", "C", "D", "F"];
        let expected_values = ["X", "Y", "3", "2", "1"];
        assert_eq!(5, rows.len());
        for i in 0..5 {
            assert_eq!(expected_keys[i], rows[i].key);
            assert_eq!(expected_values[i], rows[i].value);
        }
    }

    #[tokio::test]
    async fn test_sync_3() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments
            VALUES
            ('dev', 'A', 'X', 1697207333),
            ('dev', 'A', '1', 1697207341),
            ('dev', 'B', 'Y', 1697207331),
            ('dev', 'C', 'Z', 1697207331),
            ('dev', 'D', 'K', 1697207331);",
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = sync(&pool, "dev", "prod", false).await;
        println!("{:?}", res);
        assert!(res.is_ok());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let expected_keys = ["A", "B", "C", "D"];
        let expected_values = ["1", "Y", "Z", "K"];
        assert_eq!(4, rows.len());
        for i in 0..4 {
            assert_eq!(expected_keys[i], rows[i].key);
            assert_eq!(expected_values[i], rows[i].value);
        }
    }

    #[tokio::test]
    async fn test_sync_overwrite() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments
            VALUES
            ('prod', 'C', '3', 1697207333),
            ('prod', 'D', '2', 1697207333),
            ('prod', 'F', '1', 1697207333),
            ('dev', 'A', 'X', 1697207333),
            ('dev', 'A', '1', 1697207341),
            ('dev', 'B', 'Y', 1697207331),
            ('dev', 'C', 'Z', 1697207331),
            ('dev', 'D', 'K', 1697207331);",
        )
        .execute(&pool)
        .await
        .unwrap();

        let res = sync(&pool, "dev", "prod", true).await;
        println!("{:?}", res);
        assert!(res.is_ok());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            r"SELECT *
            FROM environments
            WHERE env = 'prod'
            GROUP BY key
            HAVING MAX(created_at)
            ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        let expected_keys = ["A", "B", "C", "D", "F"];
        let expected_values = ["1", "Y", "Z", "K", "1"];
        assert_eq!(5, rows.len());
        for i in 0..5 {
            assert_eq!(expected_keys[i], rows[i].key);
            assert_eq!(expected_values[i], rows[i].value);
        }
    }
}
