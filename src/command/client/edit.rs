use std::io::Result;

use clap::Parser;

use crate::{db::EnvelopeDb, ops};

/// Edit environment variables in editor
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to edit.
    env: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::edit(db, &self.env).await?;
        Ok(())
    }
}
