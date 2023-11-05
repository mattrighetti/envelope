use clap::Parser;

use std::io::Result;

use crate::{db::EnvelopeDb, ops, other_str_err};

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
            return Err(other_str_err!("cannot duplicate to same environment"));
        }

        ops::duplicate(db, &self.source, &self.target).await
    }
}
