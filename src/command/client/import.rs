use std::fs::File;
use std::io::{BufRead, BufReader};

use anyhow::Result;
use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Import environment variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to assign to the imported environment
    /// variables.
    env: String,

    /// Path of the file from which you want to import environment variables.
    /// Defaults to stdin if not provided.
    path: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        let reader: Box<dyn BufRead> = match &self.path {
            None => Box::new(BufReader::new(std::io::stdin())),
            Some(path) => {
                let f = File::open(path)?;
                Box::new(BufReader::new(f))
            }
        };

        ops::import(reader, &mut std::io::stdout(), db, &self.env).await?;

        Ok(())
    }
}
