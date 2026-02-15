use std::io;

use sqlx::SqlitePool;

use crate::{err, std_err};
use model::*;

pub(crate) mod model;

#[derive(Debug)]
pub struct EnvelopeDb {
    db: SqlitePool,
}

/// enum specifying truncation logic used in listing functions
pub enum Truncate {
    None,
    Max(u32),
}

#[cfg(test)]
impl EnvelopeDb {
    pub fn get_pool(&self) -> &SqlitePool {
        &self.db
    }
}

impl EnvelopeDb {
    pub(crate) fn with(db: SqlitePool) -> Self {
        EnvelopeDb { db }
    }

    /// checks if an environment exists in the database
    pub async fn env_exists(&self, env: &str) -> io::Result<bool> {
        sqlx::query_scalar(r"SELECT EXISTS(SELECT 1 FROM environments WHERE env = $1)")
            .bind(env)
            .fetch_one(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    /// Returns all active variables stored for all environments
    pub async fn get_active_kv_in_env(&self) -> io::Result<Vec<EnvironmentRow>> {
        sqlx::query_as(
            r"SELECT *
            FROM active_envs",
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

    /// soft deletes all variables in an environment
    pub async fn soft_delete_env(&self, env: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT env, key, NULL
            FROM active_envs
            WHERE env = $1",
        )
        .bind(env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// soft deletes all variables with specified key
    pub async fn soft_delete_keys(&self, key: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT env, key, NULL
            FROM active_envs
            WHERE key = UPPER($1)",
        )
        .bind(key)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// Soft deletes specified variable in passed environment
    pub async fn soft_delete_key_in_env(&self, env: &str, key: &str) -> io::Result<()> {
        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT env, key, NULL
            FROM active_envs
            WHERE
                env = $1 AND
                key = UPPER($2)",
        )
        .bind(env)
        .bind(key)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// deletes environment from database entirely
    pub async fn delete_env(&self, env: &str) -> io::Result<()> {
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

    /// duplicates `source_env` in a new environment `target_env`.
    /// In order for this to work, `tgt_env` must not be present.
    pub async fn duplicate_env(&self, source_env: &str, target_env: &str) -> io::Result<()> {
        if !self.env_exists(source_env).await? {
            return err!("source environment {} does not exist", source_env);
        }

        if self.env_exists(target_env).await? {
            return err!(
                "duplicating into existing target environment {} is not allowed",
                target_env
            );
        }

        sqlx::query(
            r"INSERT INTO environments (env, key, value)
            SELECT $2, key, value
            FROM active_envs
            WHERE env = $1",
        )
        .bind(source_env)
        .bind(target_env)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// lists all active variables in an environment
    pub async fn list_kv_in_env(&self, env: &str) -> io::Result<Vec<EnvironmentRow>> {
        self.list_kv_in_env_alt(env, Truncate::None, "da").await
    }

    /// list all key-value for specified environment, this also takes an option
    /// to truncate the max length of values returned
    pub async fn list_kv_in_env_alt(
        &self,
        env: &str,
        truncate: Truncate,
        sort: &str,
    ) -> io::Result<Vec<EnvironmentRow>> {
        let max = match truncate {
            Truncate::None => None,
            Truncate::Max(x) => Some(x),
        };

        sqlx::query_as(
            r"SELECT
                env
                , key
                , created_at
                , CASE
                    WHEN $2 IS NULL THEN value
                    ELSE substr(value, 0, $2 + 1)
                END AS value
            FROM active_envs
            WHERE env = $1
            ORDER BY
                CASE
                    WHEN $3 = 'k' THEN key END,
                CASE
                    WHEN $3 = 'kd' THEN key END DESC,
                CASE
                    WHEN $3 = 'v' THEN value END,
                CASE
                    WHEN $3 = 'vd' THEN value END DESC,
                CASE
                    WHEN $3 = 'd' THEN created_at END,
                CASE
                    WHEN $3 = 'dd' THEN created_at END DESC,
                created_at ASC",
        )
        .bind(env)
        .bind(max)
        .bind(sort)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    // lists environments present in the database. Environments that only contain
    // deletes variables will be listed as well.
    pub async fn list_environments(&self) -> io::Result<Vec<Environment>> {
        sqlx::query_as(r"SELECT DISTINCT env FROM environments ORDER BY created_at")
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    /// This returns the diff between the two specified environments
    pub async fn diff(&self, e1: &str, e2: &str) -> io::Result<Vec<EnvironmentDiff>> {
        for e in [e1, e2] {
            if !self.env_exists(e).await? {
                return err!("cannot diff non existent environment {}", e);
            }
        }

        sqlx::query_as(
            r"WITH base AS (
                SELECT key, value, env
                FROM active_envs
                WHERE env IN ($1, $2)
            )
            SELECT e1.key, e1.value, null AS diff, '+' AS type
            FROM base e1
            WHERE e1.env = $1
            AND NOT EXISTS (
                SELECT 1 FROM base e2
                WHERE e2.env = $2
                AND e2.key = e1.key
            )
            UNION ALL
            SELECT e2.key, e2.value, null AS diff, '-' AS type
            FROM base e2
            WHERE e2.env = $2
            AND NOT EXISTS (
                SELECT 1 FROM base e1
                WHERE e1.env = $1
                AND e1.key = e2.key
            )
            UNION ALL
            SELECT e1.key, e1.value, e2.value as diff, '/' AS type
            FROM base e1
            JOIN base e2 ON e1.key = e2.key
            WHERE e1.env = $1
            AND e2.env = $2
            AND e1.value != e2.value
            ORDER BY key",
        )
        .bind(e1)
        .bind(e2)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    /// Reverts a specific key-value in a specific env by deleting the most
    /// recent entry in database
    pub async fn revert(&self, env: &str, key: &str) -> io::Result<()> {
        sqlx::query(
            r"DELETE FROM environments
            WHERE
                env = $1 AND
                key = upper($2) AND
                EXISTS (
                    SELECT 1
                    FROM latest_envs
                    WHERE
                        latest_envs.env = environments.env AND
                        latest_envs.key = environments.key AND
                        latest_envs.created_at = environments.created_at AND
                        (latest_envs.value IS environments.value OR latest_envs.value = environments.value)
                )",
        )
        .bind(env)
        .bind(key)
        .execute(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    /// Lists all values for an env key pair
    pub async fn history(&self, env: &str, key: &str) -> io::Result<Vec<EnvironmentRowNullable>> {
        sqlx::query_as(
            "SELECT env, key, value, CAST(DATETIME(created_at, 'unixepoch') AS text) AS created_at
            FROM environments
            WHERE
                env = $1 AND
                key = upper($2)
            ORDER BY created_at",
        )
        .bind(env)
        .bind(key)
        .fetch_all(&self.db)
        .await
        .map_err(|e| std_err!("db error: {}", e))
    }

    #[cfg(test)]
    async fn exec(&self, sql: &str) -> io::Result<()> {
        sqlx::query(sql)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }
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

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use sqlx::Row;

    use super::*;

    #[tokio::test]
    async fn test_env_exists() {
        let db = test_db().await;
        // Insert a test environment
        db.insert("test_env", "test_key", "test_value")
            .await
            .unwrap();
        // Check if the environment exists
        assert!(db.env_exists("test_env").await.unwrap());
        // Check for a non-existent environment
        assert!(!db.env_exists("non_existent_env").await.unwrap());
    }

    #[tokio::test]
    async fn test_get_kv_in_env() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key1", "value3").await.unwrap();

        // Fetch all environment variables
        let vars = db.get_active_kv_in_env().await.unwrap();
        assert_eq!(vars.len(), 3);
        // keys are always uppercased
        let expected: HashSet<_> = [
            ("env1", "KEY1", "value1"),
            ("env1", "KEY2", "value2"),
            ("env2", "KEY1", "value3"),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.to_string()))
        .collect();

        let actual: HashSet<_> = vars
            .into_iter()
            .map(|row| (row.env, row.key, row.value))
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_insert() {
        let db = test_db().await;

        // Insert a test environment variable
        db.insert("test_env", "test_key", "test_value")
            .await
            .unwrap();

        // Fetch all environment variables
        let vars = db.get_active_kv_in_env().await.unwrap();

        let expected: HashSet<_> = [("test_env", "TEST_KEY", "test_value")]
            .into_iter()
            .map(|(e, k, v)| (e.to_string(), k.to_string(), v.to_string()))
            .collect();

        let actual: HashSet<_> = vars
            .into_iter()
            .map(|row| (row.env, row.key, row.value))
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_soft_delete_env() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key2", "value3").await.unwrap();

        // Delete environment
        db.soft_delete_env("env1").await.unwrap();

        // Query database directly to verify stored values
        let rows = sqlx::query(
            r"SELECT
                env
                , key
                , value
            FROM latest_envs",
        )
        .fetch_all(db.get_pool())
        .await
        .unwrap();

        let expected: HashSet<(String, String, Option<String>)> = [
            ("env1", "KEY1", None),
            ("env1", "KEY2", None),
            ("env2", "KEY2", Some("value3")),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.map(|x: &str| x.to_string())))
        .collect();

        let actual: HashSet<(String, String, Option<String>)> = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("env"),
                    row.get::<String, _>("key"),
                    row.get::<Option<String>, _>("value"),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_delete_env() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key2", "value3").await.unwrap();
        db.insert("env3", "key2", "value3").await.unwrap();

        // Delete environment
        db.delete_env("env2").await.unwrap();

        // Query database directly to verify stored values
        let rows = sqlx::query(
            r"SELECT
                    env
                    , key
                    , value
                FROM latest_envs",
        )
        .fetch_all(db.get_pool())
        .await
        .unwrap();

        let expected: HashSet<(String, String, Option<String>)> = [
            ("env1", "KEY1", Some("value1")),
            ("env1", "KEY2", Some("value2")),
            ("env3", "KEY2", Some("value3")),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.map(|x: &str| x.to_string())))
        .collect();

        let actual: HashSet<(String, String, Option<String>)> = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("env"),
                    row.get::<String, _>("key"),
                    row.get::<Option<String>, _>("value"),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_delete_var_for_env() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key2", "value3").await.unwrap();
        db.insert("env3", "key2", "value3").await.unwrap();

        db.soft_delete_key_in_env("env1", "key1").await.unwrap();
        db.soft_delete_key_in_env("env2", "key2").await.unwrap();

        // Query database directly to verify stored values
        let rows = sqlx::query(
            r"SELECT
                env
                , key
                , value
            FROM latest_envs",
        )
        .fetch_all(db.get_pool())
        .await
        .unwrap();

        let expected: HashSet<(String, String, Option<String>)> = [
            ("env1", "KEY2", Some("value2")),
            ("env3", "KEY2", Some("value3")),
            ("env1", "KEY1", None),
            ("env2", "KEY2", None),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.map(|x: &str| x.to_string())))
        .collect();

        let actual: HashSet<(String, String, Option<String>)> = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("env"),
                    row.get::<String, _>("key"),
                    row.get::<Option<String>, _>("value"),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_delete_var_all() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key2", "value3").await.unwrap();
        db.insert("env3", "key2", "value3").await.unwrap();

        db.soft_delete_keys("key2").await.unwrap();

        // Query database directly to verify stored values
        let rows = sqlx::query(
            r"SELECT
                    env
                    , key
                    , value
                FROM latest_envs",
        )
        .fetch_all(db.get_pool())
        .await
        .unwrap();

        let expected: HashSet<(String, String, Option<String>)> = [
            ("env1", "KEY1", Some("value1")),
            ("env1", "KEY2", None),
            ("env2", "KEY2", None),
            ("env3", "KEY2", None),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.map(|x: &str| x.to_string())))
        .collect();

        let actual: HashSet<(String, String, Option<String>)> = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("env"),
                    row.get::<String, _>("key"),
                    row.get::<Option<String>, _>("value"),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_duplicate_env() {
        let db = test_db().await;

        // Insert test data
        db.insert("env1", "key1", "value1").await.unwrap();
        db.insert("env1", "key2", "value2").await.unwrap();
        db.insert("env2", "key2", "value3").await.unwrap();
        db.insert("env3", "key2", "value3").await.unwrap();

        // Duplicate env1 -> env4
        db.duplicate_env("env1", "env4").await.unwrap();
        assert!(
            db.duplicate_env("env4", "env1").await.is_err(),
            "cannot duplicate in already present env1"
        );
        assert!(
            db.duplicate_env("env5", "env1").await.is_err(),
            "cannot duplicate from non-existent env5"
        );

        // Query database directly to verify stored values
        let rows = sqlx::query(
            r"SELECT
                env
                , key
                , value
            FROM latest_envs",
        )
        .fetch_all(db.get_pool())
        .await
        .unwrap();

        let expected: HashSet<(String, String, Option<String>)> = [
            ("env1", "KEY1", Some("value1")),
            ("env1", "KEY2", Some("value2")),
            ("env2", "KEY2", Some("value3")),
            ("env3", "KEY2", Some("value3")),
            ("env4", "KEY1", Some("value1")),
            ("env4", "KEY2", Some("value2")),
        ]
        .into_iter()
        .map(|(e, k, v)| (e.to_string(), k.to_string(), v.map(|x: &str| x.to_string())))
        .collect();

        let actual: HashSet<(String, String, Option<String>)> = rows
            .into_iter()
            .map(|row| {
                (
                    row.get::<String, _>("env"),
                    row.get::<String, _>("key"),
                    row.get::<Option<String>, _>("value"),
                )
            })
            .collect();

        assert_eq!(expected, actual);
    }

    #[tokio::test]
    async fn test_list_var_in_env() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
            VALUES
                ('env1', 'KEY1', 'value1', 0),
                ('env1', 'KEY1', NULL, 10),
                ('env1', 'KEY2', 'value2', 100),
                ('env1', 'KEY3', 'value3', 10),
                ('env1', 'KEY4', 'value4', 0),
                ('env2', 'KEY1', 'value1', 0),
                ('env2', 'KEY2', 'value2', 0),
                ('env2', 'KEY2', NULL, 10)
            ",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
            ],
            db.list_kv_in_env("env1").await.unwrap()
        );
        assert_eq!(
            vec![EnvironmentRow::from("env2", "KEY1", "value1"),],
            db.list_kv_in_env("env2").await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_list_all_var_in_env() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
            VALUES
                ('env1', 'KEY1', 'value1', 0),
                ('env1', 'KEY1', NULL, 10),
                ('env1', 'KEY2', 'value2', 100),
                ('env1', 'KEY3', 'value3', 10),
                ('env1', 'KEY4', 'value4', 0),
                ('env2', 'KEY1', 'value1', 0),
                ('env2', 'KEY2', 'value2', 0),
                ('env2', 'KEY2', NULL, 10)
            ",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "da")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "val"),
                EnvironmentRow::from("env1", "KEY3", "val"),
                EnvironmentRow::from("env1", "KEY2", "val"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::Max(3), "da")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::Max(7), "da")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", ""),
                EnvironmentRow::from("env1", "KEY3", ""),
                EnvironmentRow::from("env1", "KEY2", ""),
            ],
            db.list_kv_in_env_alt("env1", Truncate::Max(0), "da")
                .await
                .unwrap()
        );
    }

    #[tokio::test]
    async fn test_list_ordering() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
            VALUES
                ('env1', 'KEY1', 'value1', 0),
                ('env1', 'KEY2', 'value2', 100),
                ('env1', 'KEY3', 'value3', 10),
                ('env1', 'KEY4', 'value4', 1000)
            ",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY1", "value1"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY4", "value4"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "k")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY1", "value1"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "kd")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY1", "value1"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY4", "value4"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "d")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY1", "value1"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "dd")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY1", "value1"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY4", "value4"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "v")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY4", "value4"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY1", "value1"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "vd")
                .await
                .unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentRow::from("env1", "KEY1", "value1"),
                EnvironmentRow::from("env1", "KEY3", "value3"),
                EnvironmentRow::from("env1", "KEY2", "value2"),
                EnvironmentRow::from("env1", "KEY4", "value4"),
            ],
            db.list_kv_in_env_alt("env1", Truncate::None, "invalidsort")
                .await
                .unwrap(),
            "when sort key is invalid it should fallback to ordering by created_at desc"
        );
    }

    #[tokio::test]
    async fn test_list_envs() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
                VALUES
                    ('env1', 'KEY1', 'value1', 0),
                    ('env1', 'KEY1', NULL, 10),
                    ('env1', 'KEY2', 'value2', 100),
                    ('env1', 'KEY3', 'value3', 10),
                    ('env1', 'KEY4', 'value4', 0),
                    ('env2', 'KEY1', 'value1', 0),
                    ('env2', 'KEY2', 'value2', 0),
                    ('env2', 'KEY2', NULL, 10),
                    ('env3', 'KEY2', NULL, 10)
                ",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                Environment::from("env1"),
                Environment::from("env2"),
                Environment::from("env3"),
            ],
            db.list_environments().await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_diff() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
            VALUES
            ('env1', 'KEY1', 'value1', 0),
            ('env1', 'KEY1', NULL, 10),
            ('env1', 'KEY2', 'value2', 100),
            ('env1', 'KEY3', 'value3', 10),
            ('env1', 'KEY4', 'value4', 0),
            ('env2', 'KEY1', 'value1', 0),
            ('env2', 'KEY2', 'value2', 0),
            ('env1', 'MATCH', 'value1', 0),
            ('env2', 'MATCH', 'value1', 0),
            ('env1', 'NOTMATCH', 'value1', 0),
            ('env2', 'NOTMATCH', 'value2', 0),
            ('env2', 'KEY2', NULL, 10),
            ('env3', 'KEY2', NULL, 10)",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                EnvironmentDiff::InSecond("KEY1".into(), "value1".into()),
                EnvironmentDiff::InFirst("KEY2".into(), "value2".into()),
                EnvironmentDiff::InFirst("KEY3".into(), "value3".into()),
                EnvironmentDiff::InFirst("KEY4".into(), "value4".into()),
                EnvironmentDiff::Different("NOTMATCH".into(), "value1".into(), "value2".into())
            ],
            db.diff("env1", "env2").await.unwrap()
        );
        assert_eq!(
            vec![
                EnvironmentDiff::InFirst("KEY1".into(), "value1".into()),
                EnvironmentDiff::InSecond("KEY2".into(), "value2".into()),
                EnvironmentDiff::InSecond("KEY3".into(), "value3".into()),
                EnvironmentDiff::InSecond("KEY4".into(), "value4".into()),
                EnvironmentDiff::Different("NOTMATCH".into(), "value2".into(), "value1".into())
            ],
            db.diff("env2", "env1").await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_revert() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
                VALUES
                ('env1', 'KEY1', 'value1', 0),
                ('env1', 'KEY1', NULL, 10),
                ('env1', 'KEY1', 'value2', 100),
                ('env2', 'KEY1', 'value2', 10)",
        )
        .await
        .unwrap();

        assert_eq!(
            db.list_kv_in_env("env1").await.unwrap(),
            vec![EnvironmentRow::from("env1", "KEY1", "value2")]
        );
        assert_eq!(
            db.list_kv_in_env("env2").await.unwrap(),
            vec![EnvironmentRow::from("env2", "KEY1", "value2")]
        );

        // this will delete ('env1', 'KEY1', 'value2', 100)
        // and will invalidate the value
        db.revert("env1", "key1").await.unwrap();
        assert_eq!(
            db.list_kv_in_env("env1").await.unwrap(),
            vec![],
            "env1 should be empty"
        );
        assert_eq!(
            db.list_kv_in_env("env2").await.unwrap(),
            vec![EnvironmentRow::from("env2", "KEY1", "value2")],
            "env2 should remain untouched"
        );

        // this will delete ('env1', 'KEY1', NULL, 10)
        db.revert("env1", "key1").await.unwrap();
        assert_eq!(
            db.list_kv_in_env("env1").await.unwrap(),
            vec![EnvironmentRow::from("env1", "KEY1", "value1")]
        );
        assert_eq!(
            db.list_kv_in_env("env2").await.unwrap(),
            vec![EnvironmentRow::from("env2", "KEY1", "value2")],
            "env2 should remain untouched"
        );
    }

    #[tokio::test]
    async fn test_ek_history() {
        let db = test_db().await;

        db.exec(
            r"INSERT INTO environments (env, key, value, created_at)
            VALUES
                ('env1', 'KEY1', 'value1', 0),
                ('env1', 'KEY1', NULL, 10),
                ('env1', 'KEY1', 'value2', 100),
                ('env1', 'KEY2', 'value3', 10),
                ('env1', 'KEY2', 'value4', 0)
            ",
        )
        .await
        .unwrap();

        assert_eq!(
            vec![
                EnvironmentRowNullable::from("env1", "KEY1", Some("value1"), "1970-01-01 00:00:00"),
                EnvironmentRowNullable::from("env1", "KEY1", None, "1970-01-01 00:00:10"),
                EnvironmentRowNullable::from("env1", "KEY1", Some("value2"), "1970-01-01 00:01:40"),
            ],
            db.history("env1", "key1").await.unwrap()
        );
    }
}
