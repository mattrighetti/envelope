use std::io::{Result, Write};

use crate::db::{EnvelopeDb, EnvironmentRowNullable};

pub async fn history<W: Write>(
    writer: &mut W,
    db: &EnvelopeDb,
    env: &str,
    key: &str,
) -> Result<()> {
    let kvs: Vec<EnvironmentRowNullable> = db.history(env, key).await?;
    for EnvironmentRowNullable { key, value, .. } in kvs {
        if let Some(value) = value {
            writeln!(writer, "{}={}", key, value)?;
        } else {
            writeln!(writer, "{} inactive", key)?;
        }
    }

    Ok(())
}
