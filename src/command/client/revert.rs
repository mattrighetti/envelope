use std::io::Result;

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Revert environment variable to previous value
#[derive(Parser)]
pub struct Cmd {
    /// Environment of the key you wish to revert
    env: String,

    /// Environment key you wish to revert
    key: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::revert(db, &self.env, &self.key).await?;

        Ok(())
    }
}
