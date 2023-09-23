use std::io;
use std::fs;
use std::env;
use std::fs::OpenOptions;

use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to export.
    env: String,

    /// Custom output file path.
    #[arg(long, short)]
    output: Option<String>
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
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

        let mut buf = io::BufWriter::new(out);

        ops::export_dotenv(db, &self.env, &mut buf).await?;

        Ok(())
    }
}
