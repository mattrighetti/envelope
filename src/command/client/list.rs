use clap::Parser;
use std::io;
use std::io::Result;

use crate::db::{self, EnvelopeDb};
use crate::ops;

/// List saved environments and/or their variables
#[derive(Parser)]
pub struct Cmd {
    /// Environment that you wish to list.
    /// If not provided, all environments will be listed.
    env: Option<String>,

    #[arg(long, short)]
    pretty_print: bool,

    #[arg(long, short)]
    truncate: bool,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        match &self.env {
            None => ops::list_envs(&mut io::stdout(), db).await?,
            Some(env) => {
                if !self.pretty_print {
                    ops::list_raw(&mut io::stdout(), db, env).await?;
                } else {
                    let truncate = match self.truncate {
                        true => db::Truncate::Range(0, 60),
                        false => db::Truncate::None,
                    };
                    ops::list(db, env, truncate).await?;
                }
            }
        }

        Ok(())
    }
}
