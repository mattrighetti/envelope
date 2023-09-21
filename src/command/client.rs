use clap::Subcommand;

use crate::db;

mod add;
mod list;
mod import;
mod delete;
mod export;

#[derive(Subcommand)]
#[command(infer_subcommands = true)]
pub enum EnvelopeCmd {
    Import(import::Cmd),
    Add(add::Cmd),
    Delete(delete::Cmd),
    List(list::Cmd),
    Export(export::Cmd)
}

impl EnvelopeCmd {
    #[tokio::main(flavor = "current_thread")]
    pub async fn run(self) -> std::io::Result<()> {
        let db = db::init().await.unwrap();

        match self {
            Self::Delete(delete) => delete.run(&db).await?,
            Self::List(list) => list.run(&db).await?,
            Self::Add(add) => add.run(&db).await?,
            Self::Import(import) => import.run(&db).await?,
            Self::Export(export) => export.run(&db).await?
        }

        Ok(())
    }
}
