use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    env: String,
    path: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        match &self.path {
            Some(path) => ops::import_from_file(db, &self.env, path).await?,
            None => ops::import_from_stdin(db, &self.env).await?
        }

        Ok(())
    }
}
