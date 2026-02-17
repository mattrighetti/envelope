use anyhow::{Result, ensure};
use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Create a copy of another environment
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you want to duplicate
    source: String,

    /// New environment name
    target: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ensure!(
            self.source != self.target,
            "source and target environments cannot be the same"
        );

        ops::duplicate(db, &self.source, &self.target).await
    }
}
