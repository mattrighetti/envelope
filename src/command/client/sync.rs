use clap::Parser;
use sqlx::SqlitePool;

use std::io::{Error, ErrorKind, Result};

use crate::ops;

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
    pub async fn run(&self, db: &SqlitePool) -> Result<()> {
        if self.source == self.target {
            return Err(Error::new(ErrorKind::Other, "can't sync the same env"));
        }

        ops::sync(db, &self.source, &self.target, false).await
    }
}
