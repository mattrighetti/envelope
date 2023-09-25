use crate::db::EnvironmentRow;
use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use prettytable::{row, Table};

use std::io;
use std::io::{BufRead, Error, ErrorKind, Write};

pub async fn print_from_stdin() -> io::Result<()> {
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

fn query_builder(env: Option<&str>) -> QueryBuilder<Sqlite> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r"SELECT env, key, value, created_at
        FROM environments
        WHERE value NOT NULL ",
    );

    if env.is_some() {
        query_builder.push("AND env =").push_bind(env);
    }

    query_builder.push(
        r"GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key;",
    );

    query_builder
}

pub async fn print(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let envs: Vec<EnvironmentRow> = query_builder(env)
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    if !envs.is_empty() {
        Table::from(EnvRows(envs)).printstd();
    }

    Ok(())
}

pub async fn print_raw(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let envs: Vec<EnvironmentRow> = query_builder(env)
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        writeln!(io::stdout(), "{}={}", &env.key, &env.value)?;
    }

    Ok(())
}
