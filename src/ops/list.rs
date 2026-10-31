use std::io::{self, BufRead, Write};

use anyhow::{Context, Result, ensure};
use prettytable::{Table, row};

use crate::db::model::{Environment, EnvironmentRow};
use crate::db::{EnvelopeDb, Truncate};

pub async fn print_from_stdin() -> Result<()> {
    let buf = io::BufReader::new(io::stdin());
    table_from_reader(buf)?.printstd();

    Ok(())
}

fn table_from_reader<R: BufRead>(reader: R) -> Result<Table> {
    let mut table = Table::new();
    table.add_row(row!["VARIABLE", "VALUE"]);

    for line in reader.lines() {
        let line = line.context("failed to read input")?;
        if line.starts_with('#') {
            continue;
        }

        if let Some((k, v)) = line.split_once('=') {
            table.add_row(row![FrB->k, Fb->v]);
        }
    }

    Ok(table)
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

pub async fn table_list(db: &EnvelopeDb, env: &str, truncate: Truncate, sort: &str) -> Result<()> {
    ensure!(
        db.env_exists(env).await?,
        "environment '{env}' does not exist"
    );

    let envs: Vec<EnvironmentRow> = db.list_kv_in_env_alt(env, truncate, sort).await?;
    if !envs.is_empty() {
        Table::from(EnvRows(envs)).printstd();
    }

    Ok(())
}

pub async fn list_raw<W: Write>(
    writer: &mut W,
    db: &EnvelopeDb,
    env: &str,
    sort: &str,
) -> Result<()> {
    ensure!(
        db.env_exists(env).await?,
        "environment '{env}' does not exist"
    );

    let envs: Vec<EnvironmentRow> = db.list_kv_in_env_alt(env, Truncate::None, sort).await?;
    for env in envs {
        writeln!(writer, "{}={}", env.key, env.value)?;
    }

    Ok(())
}

pub async fn list_envs<W: Write>(writer: &mut W, db: &EnvelopeDb) -> Result<()> {
    let envs: Vec<Environment> = db.list_environments().await?;
    for env in envs {
        writeln!(writer, "{}", env.env)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use super::*;

    #[test]
    fn table_input_errors_are_propagated() {
        let input = BufReader::new(&b"KEY=value\n\xffinvalid"[..]);

        let error = table_from_reader(input).expect_err("invalid UTF-8 input should fail");

        assert!(error.to_string().contains("failed to read input"));
    }
}
