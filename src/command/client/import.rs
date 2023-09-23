use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to assign to the imported environment variables.
    env: String,

    /// Path of the file from where you want to import environment variables.
    /// Defaults to stdin if not provided.
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
