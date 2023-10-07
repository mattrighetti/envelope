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
    raw: bool,
}

impl Cmd {
    pub async fn run(&self, db: &SqlitePool) -> std::io::Result<()> {
        match &self.env {
            None => ops::print_envs(db).await?,
            Some(env) => {
                if self.raw {
                    ops::print_raw(db, &env).await?;
                } else {
                    ops::print(db, &env).await?;
                }
            }
        }

        Ok(())
    }
}
