use clap::Parser;
use sqlx::SqlitePool;
use std::io::{BufRead, Error, ErrorKind, Result, Write};

use crate::ops;

/// Add environment variables to a specific environment
#[derive(Parser)]
pub struct Cmd {
    /// Environment variable to which you wish to add an environment variable
    env: String,

    /// Name of the environment variable
    key: String,

    /// Read environment variable value from stdin
    #[arg(short, long)]
    stdin: bool,

    /// Value of the environment variable. Default to empty string if not provided.
    value: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> Result<()> {
        if self.stdin && self.value.is_some() {
            return Err(Error::new(
                ErrorKind::Other,
                "can't specify a value if you're reading from stdin",
            ));
        }

        let mut value = String::new();
        match self.stdin {
            true => {
                writeln!(std::io::stdout(), "Enter value for env {}: ", self.key)?;
                let stdin = std::io::stdin();
                stdin.lock().read_line(&mut value)?;
            }
            false => {
                if self.value.is_some() {
                    value = self.value.clone().unwrap();
                }
            }
        }

        ops::add_var(db, &self.env, &self.key, &value).await
    }
}
