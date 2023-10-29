use std::collections::HashSet;
use std::io::{Error, ErrorKind, Result, Write};

use sqlx::SqlitePool;

use crate::db::EnvironmentRow;

pub async fn check<W: Write>(w: &mut W, db: &SqlitePool) -> Result<()> {
    let res = check_active_envs(db).await?;
    for env in res {
        writeln!(w, "{}", env)?;
    }

    Ok(())
}

async fn check_active_envs(db: &SqlitePool) -> Result<HashSet<String>> {
    let rows = sqlx::query_as::<_, EnvironmentRow>(
        r"SELECT *
        FROM environments
        GROUP BY env, key
        HAVING MAX(created_at)",
    )
    .fetch_all(db)
    .await
    .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    // dumb implementation
    // TODO optimise this search
    let mut active = HashSet::new();
    let mut inactive = HashSet::new();
    for row in rows {
        if inactive.contains(&row.env) {
            continue;
        }

        match std::env::var(row.key) {
            Ok(val) => {
                let set = match row.value == val {
                    true => &mut active,
                    false => &mut inactive,
                };
                set.insert(row.env);
            }
            Err(_) => {
                inactive.insert(row.env);
            }
        }
    }

    let diff: HashSet<String> = active.difference(&inactive).cloned().collect();

    Ok(diff)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::db::test_db;

    #[tokio::test]
    async fn test_check_multiple_active_subset() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'ENVELOPE_TEST_MA_A', 'X'),
            ('dev', 'ENVELOPE_TEST_MA_B', 'Y'),
            ('dev', 'ENVELOPE_TEST_MA_C', 'Z'),
            ('loc', 'ENVELOPE_TEST_MA_B', 'Y'),
            ('test', 'ENVELOPE_TEST_MA_D', 'Z'),
            ('test', 'ENVELOPE_TEST_MA_E', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        std::env::set_var("ENVELOPE_TEST_MA_A", "X");
        std::env::set_var("ENVELOPE_TEST_MA_B", "Y");
        std::env::set_var("ENVELOPE_TEST_MA_C", "Z");

        let res = check_active_envs(&pool).await;
        assert!(res.is_ok());
        assert_eq!(HashSet::from(["dev".into(), "loc".into()]), res.unwrap());
    }

    #[tokio::test]
    async fn test_check_none_active() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'ENVELOPE_TEST_NA_A', 'X'),
            ('dev', 'ENVELOPE_TEST_NA_B', 'Y'),
            ('dev', 'ENVELOPE_TEST_NA_C', 'Z'),
            ('loc', 'ENVELOPE_TEST_NA_B', 'Y'),
            ('test', 'ENVELOPE_TEST_NA_D', 'Z'),
            ('test', 'ENVELOPE_TEST_NA_E', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        std::env::set_var("ENVELOPE_TEST_NA_A", "X");
        std::env::set_var("ENVELOPE_TEST_NA_C", "Z");

        let res = check_active_envs(&pool).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_check_key_present_diff_value() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'ENVELOPE_TEST_KPDV_A', 'A'),
            ('dev', 'ENVELOPE_TEST_KPDV_B', 'Y'),
            ('dev', 'ENVELOPE_TEST_KPDV_C', 'Z'),
            ('loc', 'ENVELOPE_TEST_KPDV_B', 'Y'),
            ('test', 'ENVELOPE_TEST_KPDV_D', 'A'),
            ('test', 'ENVELOPE_TEST_KPDV_E', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        std::env::set_var("ENVELOPE_TEST_KPDV_D", "X");
        std::env::set_var("ENVELOPE_TEST_KPDV_E", "K");

        let res = check_active_envs(&pool).await;
        assert!(res.is_ok());
        assert!(res.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_check_one_active() {
        let pool = test_db().await;
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            VALUES
            ('dev', 'ENVELOPE_TEST_OP_A', 'A'),
            ('dev', 'ENVELOPE_TEST_OP_B', 'Y'),
            ('dev', 'ENVELOPE_TEST_OP_C', 'Z'),
            ('loc', 'ENVELOPE_TEST_OP_B', 'Y'),
            ('test', 'ENVELOPE_TEST_OP_D', 'A'),
            ('test', 'ENVELOPE_TEST_OP_E', 'K');",
        )
        .execute(&pool)
        .await
        .unwrap();

        std::env::set_var("ENVELOPE_TEST_OP_D", "A");
        std::env::set_var("ENVELOPE_TEST_OP_E", "K");

        let res = check_active_envs(&pool).await;
        assert!(res.is_ok());
        assert_eq!(HashSet::from(["test".into()]), res.unwrap());
    }
}
