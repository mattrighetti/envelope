use clap::Parser;
use sqlx::SqlitePool;
use std::io;
use std::io::Result;

use crate::ops;

/// List saved environments and/or their variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to list.
    /// If not provided, all environments will be listed.
    env: Option<String>,

    /// List environment variables in non-tabular format.
    #[arg(long, short)]
    raw: bool,

    #[arg(long, short)]
    truncate: bool,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> Result<()> {
        match &self.env {
            None => ops::list_envs(&mut io::stdout(), db).await?,
            Some(env) => {
                if self.raw {
                    ops::list_raw(&mut io::stdout(), db, env).await?;
                } else {
                    let truncate = match self.truncate {
                        true => ops::Truncate::Range(0, 60),
                        false => ops::Truncate::None,
                    };
                    ops::list(db, env, truncate).await?;
                }
            }
        }

        Ok(())
    }
}
