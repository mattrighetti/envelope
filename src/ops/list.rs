use crate::db::{Environment, EnvironmentRow};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use prettytable::{row, Table};

use std::io;
use std::io::{BufRead, Error, ErrorKind, Result, Write};

pub async fn print_from_stdin() -> Result<()> {
    let mut table = Table::new();
    table.add_row(row!["VARIABLE", "VALUE"]);

    let buf = io::BufReader::new(io::stdin());
    for line in buf.lines() {
        if line.is_err() {
            continue;
        }

        if line.as_ref().unwrap().starts_with('#') {
            continue;
        }

        if let Some((k, v)) = line.unwrap().split_once('=') {
            table.add_row(row![FrB->k, Fb->v]);
        }
    }

    table.printstd();

    Ok(())
}

struct EnvRows(Vec<EnvironmentRow>);

impl From<EnvRows> for Table {
    fn from(value: EnvRows) -> Self {
        let mut table = Table::new();
        table.set_titles(row!["ENVIRONMENT", "VARIABLE", "VALUE"]);

        for env in value.0 {
            table.add_row(row![Fy->&env.env, Frb->&env.key, Fb->&env.value]);
        }

        table
    }
}

pub enum Truncate {
    None,
    Range(u32, u32),
}

fn query_builder(env: &str, truncate: Truncate) -> QueryBuilder<Sqlite> {
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
}

pub async fn list(pool: &SqlitePool, env: &str, truncate: Truncate) -> Result<()> {
    let envs: Vec<EnvironmentRow> = query_builder(env, truncate)
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    if !envs.is_empty() {
        Table::from(EnvRows(envs)).printstd();
    }

    Ok(())
}

pub async fn list_raw<W: Write>(writer: &mut W, pool: &SqlitePool, env: &str) -> Result<()> {
    let envs: Vec<EnvironmentRow> = query_builder(env, Truncate::None)
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        writeln!(writer, "{}={}", &env.key, &env.value)?;
    }

    Ok(())
}

pub async fn list_envs<W: Write>(writer: &mut W, pool: &SqlitePool) -> Result<()> {
    let envs: Vec<Environment> =
        sqlx::query_as::<_, Environment>("SELECT DISTINCT(env) FROM environments")
            .fetch_all(pool)
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        writeln!(writer, "{}", &env.env)?;
    }

    Ok(())
}
