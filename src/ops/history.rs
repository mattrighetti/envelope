use std::io::{Result, Write};

use crate::db::{EnvelopeDb, model::EnvironmentRowNullable};

pub async fn history<W: Write>(
    writer: &mut W,
    db: &EnvelopeDb,
    env: &str,
    key: &str,
) -> Result<()> {
    let kvs: Vec<EnvironmentRowNullable> = db.history(env, key).await?;
    for EnvironmentRowNullable {
        key,
        value,
        created_at,
        ..
    } in kvs
    {
        if let Some(value) = value {
            writeln!(writer, "{} {}={}", created_at, key, value)?;
        } else {
            writeln!(writer, "{} {} inactive", created_at, key)?;
        }
    }

    Ok(())
}
