use anyhow::{Result, bail};
use clap::Subcommand;

use crate::core::state::{EnvelopeState, UnlockedEnvelope};
use crate::db::EnvelopeDb;
use crate::{core, ops, utils};

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
mod run;

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

    /// Encrypt envelope
    Lock,

    Revert(revert::Cmd),

    Run(run::Cmd),

    /// Decrypt envelope
    Unlock,
}

impl EnvelopeCmd {
    pub async fn run(self) -> Result<()> {
        let state = core::state::detect()?;

        match (self, state) {
            // init: only valid when uninitialized
            (Self::Init, None) => {
                core::init().await?;
                Ok(())
            }
            (Self::Init, _) => bail!("envelope is already initialized"),

            // unlock: only valid when locked
            (Self::Unlock, Some(EnvelopeState::Locked(env))) => {
                let password = utils::prompt_password("Password: ")?;
                env.unlock(&password)?;
                println!("database unlocked successfully");
                Ok(())
            }
            (Self::Unlock, Some(EnvelopeState::Unlocked)) => {
                bail!("envelope is already unlocked")
            }

            // lock: only valid when unlocked
            (Self::Lock, Some(EnvelopeState::Unlocked)) => {
                let password = utils::prompt_password_confirm()?;
                let envelope = UnlockedEnvelope::open().await?;
                envelope.lock(&password)?;
                println!("database locked successfully");
                Ok(())
            }
            (Self::Lock, Some(EnvelopeState::Locked(_))) => {
                bail!("envelope is already locked")
            }

            // all other commands: only valid when unlocked
            (cmd, Some(EnvelopeState::Unlocked)) => {
                let UnlockedEnvelope { db } = UnlockedEnvelope::open().await?;
                cmd.run_with_db(&db).await
            }

            // error cases
            (_, None) => bail!("envelope is not initialized, run `envelope init` first"),
            (_, Some(EnvelopeState::Locked(_))) => {
                bail!("envelope is locked, run `envelope unlock` first")
            }
        }
    }

    async fn run_with_db(self, db: &EnvelopeDb) -> Result<()> {
        match self {
            Self::Add(add) => add.run(db).await,
            Self::Check => ops::check(&mut std::io::stdout(), db).await,
            Self::Delete(delete) => delete.run(db).await,
            Self::Drop(drop) => drop.run(db).await,
            Self::Duplicate(duplicate) => duplicate.run(db).await,
            Self::Diff(diff) => diff.run(db).await,
            Self::Edit(edit) => edit.run(db).await,
            Self::Import(import) => import.run(db).await,
            Self::History(history) => history.run(db).await,
            Self::List(list) => list.run(db).await,
            Self::Revert(revert) => revert.run(db).await,
            Self::Run(run) => run.run(db).await,
            Self::Init | Self::Lock | Self::Unlock => unreachable!(),
        }
    }
}
