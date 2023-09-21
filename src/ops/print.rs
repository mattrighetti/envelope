use sqlx::{SqlitePool, Sqlite, QueryBuilder};
use crate::db::EnvironmentRow;

use std::io;
use std::io::{Error, ErrorKind};

fn red(s: &str) -> String {
    format!("\x1b[31m{}\x1b[0m", s)
}

fn blue(s: &str) -> String {
    format!("\x1b[34m{}\x1b[0m", s)
}

fn build_query(env: Option<&str>) -> String {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r"SELECT env, key, value, created_at
        FROM environments
        WHERE 1 = 1 "
    );

    if let Some(env) = env {
        query_builder.push("AND env = ? ");
        query_builder.push_bind(env);
    }

    query_builder.push(
        r"GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key;"
    );

    query_builder.into_sql()
}

pub async fn print(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let q = build_query(env);
    let envs = sqlx::query_as::<_, EnvironmentRow>(&q)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        println!("{}  {}", red(&env.key), blue(&env.value));
    }

    Ok(())
}

pub async fn print_raw(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let q = build_query(env);
    let envs = sqlx::query_as::<_, EnvironmentRow>(&q)
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        println!("{}={}", &env.key, &env.value);
    }

    Ok(())
}
