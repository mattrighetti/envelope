use std::io::Result;
use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Display the historical values of a specific key in a given environment.
#[derive(Parser)]
pub struct Cmd {
    /// The environment to query (e.g., "staging", "production").
    env: String,

    /// The key whose value history you want to retrieve.
    key: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::history(&mut std::io::stdout(), db, &self.env, &self.key).await?;

        Ok(())
    }
}
