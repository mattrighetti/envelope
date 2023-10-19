use clap::Parser;
use sqlx::SqlitePool;
use std::io;

use crate::ops;

#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to list.
    /// If not provided, all environments will be listed.
    env: Option<String>,

    /// List environment variables in non-tabular format.
    #[arg(long, short)]
    raw: bool,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> io::Result<()> {
        match &self.env {
            None => ops::list_envs(&mut io::stdout(), db).await?,
            Some(env) => {
                if self.raw {
                    ops::list_raw(&mut io::stdout(), db, env).await?;
                } else {
                    ops::list(db, env).await?;
                }
            }
        }

        Ok(())
    }
}
