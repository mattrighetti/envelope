use std::io::Result;
use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    env: String,

    key: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::history(&mut std::io::stdout(), db, &self.env, &self.key).await?;

        Ok(())
    }
}
