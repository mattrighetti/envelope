use clap::Subcommand;
use std::io::{Error, ErrorKind, Result};

use crate::{db::EnvelopeDb, ops};

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

    /// Check which environment is currently exported
    Check,

    Add(add::Cmd),

    List(list::Cmd),

    Import(import::Cmd),

    Delete(delete::Cmd),

    Export(export::Cmd),

    Duplicate(duplicate::Cmd),

    Drop(drop::Cmd),

    Sync(sync::Cmd),
}

impl EnvelopeCmd {
    pub async fn run(self) -> Result<()> {
        let db = EnvelopeDb::load(matches!(self, Self::Init))
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

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
