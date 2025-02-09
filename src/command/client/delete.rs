use std::io::Result;

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

/// Delete environment variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to delete
    #[arg(short, long)]
    env: Option<String>,

    /// Environment variable name that you wish to delete.
    #[arg(short, long)]
    key: Option<String>,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        match (&self.env, &self.key) {
            (Some(e), Some(k)) => {
                ops::delete_var_in_env(db, e, k).await?;
            }
            (None, Some(k)) => {
                ops::delete_var_globally(db, k).await?;
            }
            (Some(e), None) => {
                ops::delete_env(db, e).await?;
            }
            _ => {}
        }

        Ok(())
    }
}
