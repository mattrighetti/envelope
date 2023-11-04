use clap::Parser;

use std::io::{Error, ErrorKind, Result};

use crate::{db::EnvelopeDb, ops};

/// Sync environment with another environment
#[derive(Parser)]
pub struct Cmd {
    /// Source environment
    source: String,

    /// Target environment
    target: String,

    /// Override values in target
    #[arg(short, long)]
    overwrite: bool,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        if self.source == self.target {
            return Err(Error::new(ErrorKind::Other, "can't sync the same env"));
        }

        ops::sync(db, &self.source, &self.target, false).await
    }
}
