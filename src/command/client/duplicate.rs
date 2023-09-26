use clap::Parser;
use sqlx::SqlitePool;
use std::io::Error;
use std::io::{self, ErrorKind};

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment that you want to duplicate
    source: String,

    /// New environment name
    target: String,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> io::Result<()> {
        if self.source == self.target {
            return Err(Error::new(
                ErrorKind::Other,
                "cannot duplicate to same environment",
            ));
        }

        ops::duplicate(db, &self.source, &self.target).await
    }
}
