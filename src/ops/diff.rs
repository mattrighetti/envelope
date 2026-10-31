use std::io::Write;

use anyhow::Result;

use crate::db::EnvelopeDb;
use crate::db::model::EnvironmentDiff;

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_DEFAULT: &str = "\x1b[0m";

pub async fn diff<W: Write>(writer: &mut W, db: &EnvelopeDb, env1: &str, env2: &str) -> Result<()> {
    let diffs = db.diff(env1, env2).await?;
    for diff in diffs {
        match diff {
            EnvironmentDiff::InFirst { key, value } => {
                writeln!(writer, "{ANSI_GREEN}+ {key}={value}{ANSI_DEFAULT}")?;
            }
            EnvironmentDiff::InSecond { key, value } => {
                writeln!(writer, "{ANSI_RED}- {key}={value}{ANSI_DEFAULT}")?;
            }
            EnvironmentDiff::Different {
                key,
                first_value,
                second_value,
            } => {
                writeln!(
                    writer,
                    "{ANSI_GRAY}/ {key}={first_value} -> {second_value}{ANSI_DEFAULT}"
                )?;
            }
        }
    }
    Ok(())
}
