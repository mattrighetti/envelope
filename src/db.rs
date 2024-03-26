use sea_query::Alias;
use sea_query::Asterisk;
use sea_query::Expr;
use sea_query::Keyword;
use sea_query::Func;
use sea_query::Order;
use sea_query::Query;
use sea_query::SqliteQueryBuilder;
use sea_query_binder::SqlxBinder;
use sqlx::SqlitePool;
use std::env;
use std::io;

use crate::std_err;

pub(crate) type EnvelopeResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug, sea_query::Iden)]
pub enum Environments {
    Table,
    Env,
    Key,
    Value,
    CreatedAt,
}

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
        let (sql, _) = Query::select()
            .from(Environments::Table)
            .column(Asterisk)
            .group_by_columns([Environments::Env, Environments::Key])
            .and_having(Expr::col(Environments::CreatedAt).max())
            .build_sqlx(SqliteQueryBuilder);

        let rows = sqlx::query_as::<_, EnvironmentRow>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(rows)
    }

    pub async fn insert(&self, env: &str, key: &str, var: &str) -> io::Result<()> {
        let (sql, values) = Query::insert()
            .into_table(Environments::Table)
            .columns([Environments::Env, Environments::Key, Environments::Value])
            .values([env.into(), Func::upper(key).into(), var.into()])
            .unwrap()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_env(&self, env: &str) -> io::Result<()> {
        let (sql, values) = Query::update()
            .table(Environments::Table)
            .values([(Environments::Value, Keyword::Null.into())])
            .and_where(Expr::col(Environments::Env).eq(env))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_var_all(&self, key: &str) -> io::Result<()> {
        let (sql, values) = Query::update()
            .table(Environments::Table)
            .values([(Environments::Value, Keyword::Null.into())])
            .and_where(Expr::col(Environments::Key).eq(key))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn delete_var_for_env(&self, env: &str, key: &str) -> io::Result<()> {
        let (sql, values) = Query::update()
            .table(Environments::Table)
            .values([(Environments::Value, Keyword::Null.into())])
            .and_where(Expr::col(Environments::Key).eq(key))
            .and_where(Expr::col(Environments::Env).eq(env))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn drop_env(&self, env: &str) -> io::Result<()> {
        let (sql, values) = Query::delete()
            .from_table(Environments::Table)
            .and_where(Expr::col(Environments::Env).eq(env))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn duplicate(&self, src_env: &str, tgt_env: &str) -> io::Result<()> {
        let select = Query::select()
            .from(Environments::Table)
            .expr(Expr::val(tgt_env))
            .column(Environments::Key)
            .column(Environments::Value)
            .and_where(Expr::col(Environments::Env).eq(src_env))
            .and_where(Expr::col(Environments::Value).is_not_null())
            .group_by_columns([Environments::Env, Environments::Key])
            .and_having(Expr::col(Environments::CreatedAt).max())
            .order_by_columns([
                (Environments::Env, Order::Desc),
                (Environments::Key, Order::Desc),
            ])
            .to_owned();

        let (sql, values) = Query::insert()
            .into_table(Environments::Table)
            .columns([Environments::Env, Environments::Key, Environments::Value])
            .select_from(select)
            .unwrap()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
            .execute(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))?;

        Ok(())
    }

    pub async fn list_var_in_env(&self, env: &str) -> io::Result<Vec<EnvironmentRow>> {
        let (sql, values) = Query::select()
            .from(Environments::Table)
            .column(Asterisk)
            .and_where(Expr::col(Environments::Env).eq(env))
            .group_by_columns([Environments::Env, Environments::Key])
            .and_having(Expr::col(Environments::CreatedAt).max())
            .order_by_columns([
                (Environments::Env, Order::Desc),
                (Environments::Key, Order::Desc),
            ])
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with(&sql, values)
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn sync(&self, src_env: &str, tgt_env: &str, overwrite: bool) -> io::Result<()> {
        let mut select = Query::select()
            .from(Environments::Table)
            .expr(Expr::val(tgt_env))
            .column(Environments::Key)
            .column(Environments::Value)
            .and_where(Expr::col(Environments::Env).eq(src_env))
            .group_by_col(Environments::Key)
            .and_having(Expr::col(Environments::CreatedAt).max())
            .order_by_columns([
                (Environments::Env, Order::Desc),
                (Environments::Key, Order::Desc),
            ])
            .to_owned();

        if !overwrite {
            select.and_where(
                Expr::col(Environments::Key).not_in_subquery(
                    Query::select()
                        .from(Environments::Table)
                        .column(Environments::Key)
                        .and_where(Expr::col(Environments::Env).eq(tgt_env))
                        .group_by_col(Environments::Key)
                        .and_having(Expr::col(Environments::CreatedAt).max())
                        .to_owned(),
                ),
            );
        }

        let (sql, values) = Query::insert()
            .into_table(Environments::Table)
            .columns([Environments::Env, Environments::Key, Environments::Value])
            .select_from(select)
            .unwrap()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_with(&sql, values)
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
        let mut select = Query::select()
            .from(Environments::Table)
            .column(Environments::Env)
            .column(Environments::Key)
            .column(Environments::CreatedAt)
            .and_where(Expr::col(Environments::Value).is_not_null())
            .and_where(Expr::col(Environments::Env).eq(env))
            .group_by_col(Environments::Key)
            .and_having(Expr::col(Environments::CreatedAt).max())
            .order_by_columns([
                (Environments::Env, Order::Desc),
                (Environments::Key, Order::Desc),
            ])
            .to_owned();

        match truncate {
            Truncate::None => select.column(Environments::Value),
            Truncate::Range(x, y) => select.expr(
                Expr::cust(format!("substr(value, {}, {}) as value", x, y))
                    .cast_as(Alias::new("value")),
            ),
        };

        let (sql, values) = select.build_sqlx(SqliteQueryBuilder);
        sqlx::query_as_with(&sql, values)
            .fetch_all(&self.db)
            .await
            .map_err(|e| std_err!("db error: {}", e))
    }

    pub async fn list_environments(&self) -> io::Result<Vec<Environment>> {
        let (sql, _) = Query::select()
            .from(Environments::Table)
            .column(Environments::Env)
            .distinct()
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as(&sql)
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
