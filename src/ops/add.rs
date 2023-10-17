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

pub async fn import<W: Write, R: BufRead>(
    reader: R,
    writer: &mut W,
    pool: &SqlitePool,
    env: &str,
) -> io::Result<()> {
    for line in reader.lines() {
        if line.is_err() {
            continue;
        }

        if line.as_ref().unwrap().starts_with('#') {
            writeln!(writer, "skipping {}", line.unwrap())?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{test_db, EnvironmentRow};
    use std::io::BufReader;

    fn stdin_input(s: &str) -> BufReader<&[u8]> {
        BufReader::new(s.as_bytes())
    }

    #[tokio::test]
    async fn test_import() {
        let pool = test_db().await;
        let mut output: Vec<u8> = Vec::new();

        let res = import(
            stdin_input("key1=value1\nkey2=value2"),
            &mut output,
            &pool,
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

    #[tokio::test]
    async fn test_import_none() {
        let pool = test_db().await;
        let mut output: Vec<u8> = Vec::new();

        let res = import(stdin_input("# key1=value1"), &mut output, &pool, "prod").await;
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
}
