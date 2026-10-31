use std::io::{BufRead, Write};

use anyhow::{Context, Result, ensure};

use crate::db::EnvelopeDb;

/// Adds a single key-value element to the database
///
/// If the value of v is None, an empty string is inserted
pub async fn add_var(db: &EnvelopeDb, env: &str, k: &str, v: &str) -> Result<()> {
    ensure!(!k.starts_with('#'), "variable name cannot start with '#'");

    db.insert(env, k, v).await?;

    Ok(())
}

pub async fn import<W: Write, R: BufRead>(
    reader: R,
    writer: &mut W,
    db: &EnvelopeDb,
    env: &str,
) -> Result<()> {
    let mut variables = Vec::new();

    for line in reader.lines() {
        let line = line.context("failed to read import input")?;
        if line.starts_with('#') {
            writeln!(writer, "skipping {line}")?;
            continue;
        }

        if let Some((k, v)) = line.split_once('=') {
            variables.push((k.to_owned(), v.to_owned()));
        } else {
            writeln!(writer, "invalid {line}, skipping")?;
        }
    }

    db.apply_batch(env, &[], &variables).await
}

#[cfg(test)]
mod test {
    use std::io::BufReader;

    use sqlx::{Row, SqlitePool};

    use super::*;
    use crate::db::EnvelopeDb;
    use crate::db::model::EnvironmentRow;

    pub fn stdin_input(s: &str) -> BufReader<&[u8]> {
        BufReader::new(s.as_bytes())
    }

    #[sqlx::test]
    async fn test_import(pool: SqlitePool) {
        let db = EnvelopeDb::with(pool.clone());
        let mut output: Vec<u8> = Vec::new();

        let res = import(
            stdin_input("key1=value1\nkey2=value2"),
            &mut output,
            &db,
            "prod",
        )
        .await;
        assert!(res.is_ok());
        assert!(output.is_empty());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(2, rows.len());
        let key_expected = ["KEY1", "KEY2"];
        let value_expected = ["value1", "value2"];
        for (i, row) in rows.into_iter().enumerate() {
            assert_eq!(key_expected[i], row.key);
            assert_eq!(value_expected[i], row.value);
        }
    }

    #[sqlx::test]
    async fn test_import_none(pool: SqlitePool) {
        let db = EnvelopeDb::with(pool.clone());

        let mut output: Vec<u8> = Vec::new();

        let res = import(stdin_input("# key1=value1"), &mut output, &db, "prod").await;
        assert!(res.is_ok());
        assert!(!output.is_empty());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(rows.is_empty());

        let output = String::from_utf8(output).unwrap();
        assert_eq!("skipping # key1=value1\n", output.as_str());
    }

    #[sqlx::test]
    async fn test_mul_import(pool: SqlitePool) {
        let db = EnvelopeDb::with(pool.clone());

        let mut output: Vec<u8> = Vec::new();

        let res = import(
            stdin_input("#k=v\n#invalid-value\nkey value\nkey1=val1\nkey2=val2"),
            &mut output,
            &db,
            "prod",
        )
        .await;

        assert!(res.is_ok());

        let rows = sqlx::query_as::<_, EnvironmentRow>(
            "SELECT * FROM environments WHERE env = 'prod' ORDER BY key",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(2, rows.len());

        let output = String::from_utf8(output).unwrap();
        assert_eq!(
            "skipping #k=v\nskipping #invalid-value\ninvalid key value, skipping\n",
            output
        );
    }

    #[sqlx::test]
    async fn test_import_propagates_input_errors(pool: SqlitePool) {
        let db = EnvelopeDb::with(pool);
        let mut output = Vec::new();
        let input = BufReader::new(&b"KEY=value\n\xffinvalid"[..]);

        let error = import(input, &mut output, &db, "prod")
            .await
            .expect_err("invalid UTF-8 input should fail the import");

        assert!(error.to_string().contains("failed to read import input"));
    }

    #[sqlx::test]
    async fn test_import_rolls_back_on_database_error(pool: SqlitePool) {
        // Reject the second row to verify the first row is rolled back too.
        sqlx::query(
            "CREATE TRIGGER reject_bad_key BEFORE INSERT ON environments
             WHEN NEW.key = 'BAD' BEGIN SELECT RAISE(ABORT, 'rejected key'); END",
        )
        .execute(&pool)
        .await
        .unwrap();
        let db = EnvelopeDb::with(pool.clone());
        let mut output = Vec::new();

        let result = import(
            stdin_input("GOOD=value\nBAD=value"),
            &mut output,
            &db,
            "prod",
        )
        .await;

        assert!(result.is_err());
        let rows = sqlx::query("SELECT COUNT(*) AS count FROM environments WHERE env = 'prod'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(rows.get::<i64, _>("count"), 0);
    }
}
