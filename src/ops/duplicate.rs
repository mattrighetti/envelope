use std::io::Result;

use crate::db::EnvelopeDb;

pub async fn duplicate(db: &EnvelopeDb, source: &str, target: &str) -> Result<()> {
    db.duplicate(source, target).await
}
