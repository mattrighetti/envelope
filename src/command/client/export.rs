use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{BufWriter, Result};

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Export environment variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to export.
    env: String,

    /// Custom output file path.
    #[arg(long, short)]
    output: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        let mut opts = OpenOptions::new();
        opts.create(true);
        opts.write(true);

        let out: fs::File = match &self.output {
            Some(out) => opts.open(out)?,
            None => {
                let dotenv = env::current_dir()?.join(".env");
                opts.open(dotenv)?
            }
        };

        let mut buf = BufWriter::new(out);

        ops::export_dotenv(db, &self.env, &mut buf).await?;

        Ok(())
    }
}
