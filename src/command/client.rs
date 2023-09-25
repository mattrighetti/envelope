use clap::Subcommand;

use crate::db;

mod add;
mod delete;
mod export;
mod import;
mod list;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub enum EnvelopeCmd {
    /// Add environment variables
    Add(add::Cmd),

    /// List environment variables
    List(list::Cmd),

    /// Import environment variables
    Import(import::Cmd),

    /// Delete environment variables
    Delete(delete::Cmd),

    /// Export environment variables
    Export(export::Cmd),
}

impl EnvelopeCmd {
    pub async fn run(self) -> std::io::Result<()> {
        let db = db::init().await.unwrap();

        match self {
            Self::Delete(delete) => delete.run(&db).await?,
            Self::List(list) => list.run(&db).await?,
            Self::Add(add) => add.run(&db).await?,
            Self::Import(import) => import.run(&db).await?,
            Self::Export(export) => export.run(&db).await?,
        }

        Ok(())
    }
}
