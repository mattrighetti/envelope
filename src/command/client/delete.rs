use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    env: String
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        ops::delete_env(db, &self.env).await?;

        Ok(())
    }
}
