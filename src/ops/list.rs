use std::io;
use std::io::{BufRead, Write};

use anyhow::{Result, ensure};
use prettytable::{Table, row};

use crate::db::model::{Environment, EnvironmentRow};
use crate::db::{EnvelopeDb, Truncate};

mod format;

pub use format::RawOutputFormat;

use self::format::{write_nu_record, write_raw_entry};

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
    output_format: RawOutputFormat,
) -> Result<()> {
    ensure!(
        db.env_exists(env).await?,
        "environment '{env}' does not exist"
    );

    let envs: Vec<EnvironmentRow> = db.list_kv_in_env_alt(env, Truncate::None, sort).await?;

    if output_format == RawOutputFormat::Nu {
        return write_nu_record(writer, &envs);
    }

    for env in envs {
        write_raw_entry(writer, &env.key, &env.value, output_format)?;
    }

    Ok(())
}

pub async fn list_envs<W: Write>(writer: &mut W, db: &EnvelopeDb) -> Result<()> {
    let envs: Vec<Environment> = db.list_environments().await?;
    for env in envs {
        writeln!(writer, "{}", &env.env)?;
    }

    Ok(())
}
