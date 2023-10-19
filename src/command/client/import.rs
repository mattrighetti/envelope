use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};

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
        let reader: Box<dyn BufRead> = match &self.path {
            None => Box::new(BufReader::new(io::stdin())),
            Some(path) => {
                let f = File::open(path)?;
                Box::new(BufReader::new(f))
            }
        };

        ops::import(reader, &mut io::stdout(), db, &self.env).await?;

        Ok(())
    }
}
