use clap::Parser;
use sqlx::SqlitePool;

use std::io::{Error, ErrorKind, Result};

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
    pub async fn run(&self, db: &SqlitePool) -> Result<()> {
        if self.source == self.target {
            return Err(Error::new(
                ErrorKind::Other,
                "cannot duplicate to same environment",
            ));
        }

        ops::duplicate(db, &self.source, &self.target).await
    }
}
