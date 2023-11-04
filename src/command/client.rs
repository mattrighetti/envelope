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
    Add(add::Cmd),

    /// Check which environment is currently exported
    Check,

    Delete(delete::Cmd),

    Drop(drop::Cmd),

    Duplicate(duplicate::Cmd),

    Export(export::Cmd),

    /// Initialize envelope
    Init,

    Import(import::Cmd),

    List(list::Cmd),

    Sync(sync::Cmd),
}

impl EnvelopeCmd {
    pub async fn run(self) -> Result<()> {
        let db = EnvelopeDb::load(matches!(self, Self::Init))
            .await
            .map_err(|e| Error::new(ErrorKind::Other, e.to_string()))?;

        match self {
            Self::Add(add) => add.run(&db).await?,
            Self::Check => ops::check(&mut std::io::stdout(), &db).await?,
            Self::Delete(delete) => delete.run(&db).await?,
            Self::Drop(drop) => drop.run(&db).await?,
            Self::Duplicate(duplicate) => duplicate.run(&db).await?,
            Self::Export(export) => export.run(&db).await?,
            Self::Import(import) => import.run(&db).await?,
            Self::List(list) => list.run(&db).await?,
            Self::Sync(sync) => sync.run(&db).await?,
            _ => {}
        }

        Ok(())
    }
}
