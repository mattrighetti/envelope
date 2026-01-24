use std::io::{Result, Write};

use crate::db::{EnvelopeDb, model::EnvironmentDiff};

const ANSI_RED: &str = "\x1b[31m";
const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_GRAY: &str = "\x1b[90m";
const ANSI_DEFAULT: &str = "\x1b[0m";

pub async fn diff<W: Write>(writer: &mut W, db: &EnvelopeDb, env1: &str, env2: &str) -> Result<()> {
    let diffs = db.diff(env1, env2).await?;
    for diff in diffs {
        match diff {
            EnvironmentDiff::InFirst(k, v) => {
                writeln!(writer, "{ANSI_GREEN}+ {}={}{ANSI_DEFAULT}", k, v)?;
            }
            EnvironmentDiff::InSecond(k, v) => {
                writeln!(writer, "{ANSI_RED}- {}={}{ANSI_DEFAULT}", k, v)?;
            }
            EnvironmentDiff::Different(k, v1, v2) => {
                writeln!(writer, "{ANSI_GRAY}/ {}={} -> {}{ANSI_DEFAULT}", k, v1, v2)?;
            }
        }
    }
    Ok(())
}
