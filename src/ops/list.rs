use sqlx::{SqlitePool, Sqlite, QueryBuilder};
use crate::db::EnvironmentRow;

use prettytable::{Table, row};

use std::io;
use std::io::{Error, ErrorKind, Write, BufRead};


pub async fn print_from_stdin() -> io::Result<()> {
    let mut table = Table::new();
    table.add_row(row!["VARIABLE", "VALUE"]);

    let buf = io::BufReader::new(io::stdin());
    for line in buf.lines() {
        if line.is_err() {
            continue
        }

        if line.as_ref().unwrap().starts_with('#') {
            continue
        }

        if let Some((k, v)) = line.unwrap().split_once('=') {
            table.add_row(row![FrB->k, Fb->v]);
        }
    }

    table.printstd();

    Ok(())
}

pub async fn print(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r"SELECT env, key, value, created_at
        FROM environments
        WHERE value NOT NULL "
    );

    if env.is_some() {
        query_builder.push("AND env =").push_bind(env);
    }

    query_builder.push(
        r"GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key;"
    );

    let envs: Vec<EnvironmentRow> = query_builder.build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    if !envs.is_empty() {
        let mut table = Table::new();
        table.add_row(row!["ENVIRONMENT", "VARIABLE", "VALUE"]);

        for env in envs {
            table.add_row(row![Fy->&env.env, Frb->&env.key, Fb->&env.value]);
        }

        table.printstd();
    }

    Ok(())
}

pub async fn print_raw(pool: &SqlitePool, env: Option<&str>) -> io::Result<()> {
    let mut query_builder: QueryBuilder<Sqlite> = QueryBuilder::new(
        r"SELECT env, key, value, created_at
        FROM environments
        WHERE value NOT NULL "
    );

    if env.is_some() {
        query_builder.push("AND env =").push_bind(env);
    }

    query_builder.push(
        r"GROUP BY env, key
        HAVING MAX(created_at)
        ORDER BY env, key;"
    );

    let envs: Vec<EnvironmentRow> = query_builder.build_query_as()
        .fetch_all(pool)
        .await
        .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

    for env in envs {
        writeln!(io::stdout(), "{}={}", &env.key, &env.value)?;
    }

    Ok(())
}