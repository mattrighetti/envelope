use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::env;
use std::io;

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
    pub created_at: i32,
}

impl EnvironmentRow {
    pub fn kv_string(&self) -> String {
        format!("{}={}", self.key, self.value)
    }
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

    pub async fn get_all_env_vars(&self) -> io::Result<Vec<EnvironmentRow>> {
        let rows = sqlx::query_as::<_, EnvironmentRow>(
            r"SELECT *
            FROM environments
            GROUP BY env, key
            HAVING MAX(created_at)",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(rows)
    }

    pub async fn insert(&self, env: &str, key: &str, var: &str) -> io::Result<()> {
        sqlx::query("INSERT INTO environments(env,key,value) VALUES (?, upper(?), ?);")
            .bind(env)
            .bind(key)
            .bind(var)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_env(&self, env: &str) -> io::Result<()> {
        sqlx::query("UPDATE environments SET value = NULL WHERE env = ?")
            .bind(env)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_var_all(&self, key: &str) -> io::Result<()> {
        sqlx::query("UPDATE environments SET value = NULL WHERE key = ?")
            .bind(key)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_var_for_env(&self, env: &str, key: &str) -> io::Result<()> {
        sqlx::query("UPDATE environments SET value = NULL WHERE env = ? AND key = ?")
            .bind(env)
            .bind(key)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn drop_env(&self, env: &str) -> io::Result<()> {
        sqlx::query("DELETE FROM environments WHERE env = ?")
            .bind(env)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn duplicate(&self, src_env: &str, tgt_env: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments(env,key,value)
            SELECT ?2, key, value
            FROM environments WHERE env = ?1 AND value NOT NULL
            GROUP BY env, key
            HAVING MAX(created_at)
            ORDER BY env, key;",
        )
        .bind(src_env)
        .bind(tgt_env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn list_var_in_env(&self, env: &str) -> io::Result<Vec<EnvironmentRow>> {
        sqlx::query_as::<_, EnvironmentRow>(
            r"SELECT env, key, value, created_at
            FROM environments
            WHERE env = ?
            GROUP BY env, key
            HAVING MAX(created_at)
            ORDER BY env, key",
        )
        .bind(env)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn sync(&self, src_env: &str, tgt_env: &str, overwrite: bool) -> io::Result<()> {
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
            .build()
            .bind(src_env)
            .bind(tgt_env)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn list_all_var_in_env(
        &self,
        env: &str,
        truncate: Truncate,
    ) -> io::Result<Vec<EnvironmentRow>> {
        let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(r"SELECT env, key, ");

        match truncate {
            Truncate::None => query_builder.push("value"),
            Truncate::Range(x, y) => {
                query_builder.push(format!("substr(value, {}, {}) as value", x, y))
            }
        };

        query_builder.push(
            r", created_at
            FROM environments
            WHERE value NOT NULL ",
        );
        query_builder.push("AND env =").push_bind(env);
        query_builder.push(
            r"GROUP BY env, key
            HAVING MAX(created_at)
            ORDER BY env, key;",
        );

        query_builder
            .build_query_as()
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn list_environments(&self) -> io::Result<Vec<Environment>> {
        sqlx::query_as::<_, Environment>("SELECT DISTINCT(env) FROM environments")
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
