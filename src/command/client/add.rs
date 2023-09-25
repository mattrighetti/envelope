use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment variable to which you wish to add an environment variable
    env: String,

    /// Name of the environment variable
    key: String,

    /// Value of the environment variable. Default to empty string if not provided.
    value: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        let value = match &self.value {
            None => "",
            Some(s) => s,
        };

        ops::add_var(db, &self.env, &self.key, value).await
    }
}
