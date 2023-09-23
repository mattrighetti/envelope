use clap::Parser;
use sqlx::SqlitePool;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to list.
    /// If not provided, all environments will be listed.
    env: Option<String>,

    /// List environment variables in non-tabular format.
    #[arg(long, short)]
    raw: bool
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        if self.raw {
            ops::print_raw(db, self.env.as_deref()).await?;
        } else {
            ops::print(db, self.env.as_deref()).await?;
        }

        Ok(())
    }
}
