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
            EnvironmentDiff::InFirst(k, v) => {
                writeln!(writer, "{ANSI_GREEN}+ {k}={v}{ANSI_DEFAULT}")?;
            }
            EnvironmentDiff::InSecond(k, v) => {
                writeln!(writer, "{ANSI_RED}- {k}={v}{ANSI_DEFAULT}")?;
            }
            EnvironmentDiff::Different(k, v1, v2) => {
                writeln!(writer, "{ANSI_GRAY}/ {k}={v1} -> {v2}{ANSI_DEFAULT}")?;
            }
        }
    }
    Ok(())
}
