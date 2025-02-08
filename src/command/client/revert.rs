use clap::Parser;
use std::io::Result;

use crate::{db::EnvelopeDb, ops};

/// Revert environment variable
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
