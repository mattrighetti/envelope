use std::io::Result;

use clap::Parser;

use crate::{db::EnvelopeDb, ops};

/// Drop environment
#[derive(Parser)]
pub struct Cmd {
    /// Environment to drop
    env: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::drop(db, &self.env).await
    }
}
