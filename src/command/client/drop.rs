use std::io;

use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment to drop
    env: String,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> io::Result<()> {
        ops::drop(db, &self.env).await
    }
}
