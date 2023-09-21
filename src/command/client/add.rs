use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    env: String,
    key: String,
    value: Option<String>
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        let value = match &self.value {
            None => "",
            Some(s) => s
        };

        ops::add_var(db, &self.env, &self.key, value).await
    }
}
