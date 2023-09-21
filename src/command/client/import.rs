use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Debug, Parser)]
pub struct Cmd {
    env: String,
    path: String
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        ops::import_from(db, &self.env, &self.path).await?;

        Ok(())
    }
}
