use clap::Subcommand;
use std::io::{Error, ErrorKind};

use crate::{db, ops};

mod add;
mod delete;
mod drop;
mod duplicate;
mod export;
mod import;
mod list;
mod sync;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub enum EnvelopeCmd {
    /// Initialize envelope
    Init,

    /// Add environment variables
    Add(add::Cmd),

    /// Check which environment is currently exported
    Check,

    /// List environment variables
    List(list::Cmd),

    /// Import environment variables
    Import(import::Cmd),

    /// Delete environment variables
    Delete(delete::Cmd),

    /// Export environment variables
    Export(export::Cmd),

    /// Duplicate environments
    Duplicate(duplicate::Cmd),

    /// Drop environment
    Drop(drop::Cmd),

    /// Syncs environments
    Sync(sync::Cmd),
}

impl EnvelopeCmd {
    pub async fn run(self) -> std::io::Result<()> {
        if !db::is_present() && !matches!(self, Self::Init) {
            return Err(Error::new(ErrorKind::Other, "envelope is not initialized"));
        }

        let db = db::init().await.unwrap();

        match self {
            Self::Delete(delete) => delete.run(&db).await?,
            Self::List(list) => list.run(&db).await?,
            Self::Add(add) => add.run(&db).await?,
            Self::Import(import) => import.run(&db).await?,
            Self::Export(export) => export.run(&db).await?,
            Self::Duplicate(duplicate) => duplicate.run(&db).await?,
            Self::Drop(drop) => drop.run(&db).await?,
            Self::Sync(sync) => sync.run(&db).await?,
            Self::Check => ops::check(&mut std::io::stdout(), &db).await?,
            _ => {}
        }

        Ok(())
    }
}
