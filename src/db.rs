use sqlx::SqlitePool;
use std::{env, io};

use crate::std_err;

pub(crate) type EnvelopeResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Environment {
    pub env: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnvironmentRow {
    pub env: String,
    pub key: String,
    pub value: String,
}

pub fn is_present() -> bool {
    if let Ok(current_dir) = env::current_dir() {
        let envelope_fs = current_dir.join(".envelope");
        return envelope_fs.is_file();
    }

    false
}

/// Checks if an `.envelope` file is present in the current directory,
/// if it is nothing is done and an error in returned, otherwise a new envelope
/// database will get created
pub async fn init() -> EnvelopeResult<SqlitePool> {
    let envelope_fs = env::current_dir()?.join(".envelope");
    let db_path = envelope_fs.into_os_string().into_string().unwrap();
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=rwc", db_path))
        .await
        .map_err(|err| format!("{}\nfile: {}", err, db_path))?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

#[derive(Debug)]
pub struct EnvelopeDb {
    db: SqlitePool,
}

#[cfg(test)]
impl EnvelopeDb {
    pub(crate) fn with(pool: SqlitePool) -> Self {
        EnvelopeDb { db: pool }
    }

    pub fn get_pool(&self) -> &SqlitePool {
        &self.db
    }
}

impl EnvelopeDb {
    pub async fn init() -> EnvelopeResult<Self> {
        let db = init().await?;

        Ok(EnvelopeDb { db })
    }

    pub async fn load(init: bool) -> EnvelopeResult<Self> {
        if !is_present() && !init {
            return Err("envelope is not initialized in current directory".into());
        }

        EnvelopeDb::init().await
    }

    /// checks if an environment exists in the database
    pub async fn check_env_exists(&self, env: &str) -> io::Result<bool> {
        sqlx::query_scalar(r"SELECT EXISTS(SELECT 1 FROM environments WHERE env = $1)")
            .bind(env)
            .fetch_one(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn get_all_env_vars(&self) -> io::Result<Vec<EnvironmentRow>> {
        sqlx::query_as(
            r"SELECT *
            FROM environments
            GROUP BY env, key
            HAVING MAX(created_at)",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    /// inserts `key` and `value` to environment `env`
    pub async fn insert(&self, env: &str, key: &str, var: &str) -> io::Result<()> {
        sqlx::query(r"INSERT INTO environments (env, key, value) VALUES ($1, upper($2), $3)")
            .bind(env)
            .bind(key)
            .bind(var)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// soft deletes all variables in an environment by setting all their
    /// values to NULL
    pub async fn delete_env(&self, env: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT env, key, NULL
            FROM environments
            WHERE
                env = $1 AND
                value IS NOT NULL
            GROUP BY env, key",
        )
        .bind(env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// soft deletes all variables with key `key`
    pub async fn delete_var_all(&self, key: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT env, key, NULL
            FROM environments
            WHERE
                key = $1 AND
                value IS NOT NULL
            GROUP BY env, key",
        )
        .bind(key)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_var_for_env(&self, env: &str, key: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, values)
            SELECT env, key, NULL
            FROM environments
            WHERE
                env = $1 AND
                key = $2 AND
                value IS NOT NULL
            GROUP BY env, key",
        )
        .bind(env)
        .bind(key)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// deletes environment from database entirely
    pub async fn drop_env(&self, env: &str) -> io::Result<()> {
        sqlx::query(
            r"DELETE
            FROM environments
            WHERE env = $1",
        )
        .bind(env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// duplicates `src_env` in a new environment `tgt_env`
    pub async fn duplicate(&self, source_env: &str, target_env: &str) -> io::Result<()> {
        sqlx::query(
            r"
            WITH sub AS (
                SELECT *
                FROM environments
                WHERE
                    env = $1
                GROUP BY env, key
                HAVING MAX(created_at)
            )
            INSERT INTO environments (env, key, value)
            SELECT $2, key, value
            FROM sub
            WHERE
                env = $1 AND
                value IS NOT NULL
            GROUP BY env, key
            HAVING MAX(created_at)
            ORDER BY env DESC, key DESC",
        )
        .bind(source_env)
        .bind(target_env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn list_var_in_env(&self, env: &str) -> io::Result<Vec<EnvironmentRow>> {
        sqlx::query_as(
            r"WITH envs AS (
                SELECT *
                FROM environments
                WHERE
                    env = $1
                GROUP BY env, key
                HAVING MAX(created_at)
            )
            SELECT *
            FROM envs
            WHERE
                value IS NOT NULL
            ORDER BY env DESC, key DESC",
        )
        .bind(env)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn list_all_var_in_env(
        &self,
        env: &str,
        truncate: Truncate,
    ) -> io::Result<Vec<EnvironmentRow>> {
        let (x, y) = match truncate {
            Truncate::None => (None, None),
            Truncate::Range(x, y) => (Some(x), Some(y)),
        };

        sqlx::query_as(
            r"WITH sub AS (
                SELECT *
                FROM environments
                WHERE
                    env = $1
                GROUP BY env, key
                HAVING MAX(created_at)
            )
            SELECT
                env
                , key
                , created_at
                CASE
                    WHEN $2 IS NULL AND $3 IS NULL THEN value
                    ELSE substr(value, $2, $3)
                END AS value
            FROM sub
            WHERE
                value IS NOT NULL AND
                env = $1
            GROUP BY key
            HAVING MAX(created_at)
            ORDER BY env DESC, key DESC",
        )
        .bind(env)
        .bind(x)
        .bind(y)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    // lists environments present in the database. Environments that only contain deletes variables
    // will be listed as well.
    pub async fn list_environments(&self) -> io::Result<Vec<Environment>> {
        sqlx::query_as(r"SELECT DISTINCT env FROM environments")
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }
}

pub enum Truncate {
    None,
    Range(u32, u32),
}

#[cfg(test)]
pub async fn test_db() -> EnvelopeDb {
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect(":memory:")
        .await
        .expect("cannot connect to db");

    sqlx::migrate!("./migrations").run(&pool).await.unwrap();

    EnvelopeDb::with(pool)
}
