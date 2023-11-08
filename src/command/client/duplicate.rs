use clap::Parser;

use std::io::Result;

use crate::{db::EnvelopeDb, err, ops};

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
        if self.source == self.target {
            return Err(err!("cannot duplicate to same environment"));
        }

        ops::duplicate(db, &self.source, &self.target).await
    }
}
