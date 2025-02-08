use std::io::Result;

use crate::db::EnvelopeDb;

pub async fn revert(db: &EnvelopeDb, env: &str, key: &str) -> Result<()> {
    db.revert(env, key).await?;

    Ok(())
}
