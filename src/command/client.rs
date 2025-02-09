use std::io::Result;

use clap::Subcommand;

use crate::db::EnvelopeDb;
use crate::{ops, std_err};

mod add;
mod delete;
mod diff;
mod drop;
mod duplicate;
mod edit;
mod history;
mod import;
mod list;
mod revert;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub enum EnvelopeCmd {
    Add(add::Cmd),

    /// Check which environment is currently exported
    Check,

    Delete(delete::Cmd),

    Drop(drop::Cmd),

    Duplicate(duplicate::Cmd),

    Diff(diff::Cmd),

    Edit(edit::Cmd),

    History(history::Cmd),

    /// Initialize envelope
    Init,

    Import(import::Cmd),

    List(list::Cmd),

    Revert(revert::Cmd),
}

impl EnvelopeCmd {
    pub async fn run(self) -> Result<()> {
        let db = EnvelopeDb::load(matches!(self, Self::Init))
            .await
            .map_err(|e| std_err!("{}", e.to_string()))?;

        match self {
            Self::Add(add) => add.run(&db).await?,
            Self::Check => ops::check(&mut std::io::stdout(), &db).await?,
            Self::Delete(delete) => delete.run(&db).await?,
            Self::Drop(drop) => drop.run(&db).await?,
            Self::Duplicate(duplicate) => duplicate.run(&db).await?,
            Self::Diff(diff) => diff.run(&db).await?,
            Self::Edit(edit) => edit.run(&db).await?,
            Self::Import(import) => import.run(&db).await?,
            Self::History(history) => history.run(&db).await?,
            Self::List(list) => list.run(&db).await?,
            Self::Revert(revert) => revert.run(&db).await?,
            _ => {}
        }

        Ok(())
    }
}
