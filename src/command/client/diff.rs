use std::io::Result;

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Diff two environments
#[derive(Parser)]
pub struct Cmd {
    env1: String,
    env2: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::diff(&mut std::io::stdout(), db, &self.env1, &self.env2).await?;

        Ok(())
    }
}
