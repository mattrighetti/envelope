use std::io::Result;

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::ops;

#[derive(Parser)]
#[command(
    about = "Diff two environments",
    long_about = "Diff two environments\n
This command will list all the variables that differ between source and target environments, order matters.
- Variables present in env1 but not in env2 will be shown in green.
- Variables not present in env1 but present in env2 will be shown in red.
- Variables both present in env1 and env2 but with a different value will be shown in gray, first value being the value in env1, second value is the value found in env2."
)]
pub struct Cmd {
    /// Source environment
    env1: String,
    /// Target environment
    env2: String,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        ops::diff(&mut std::io::stdout(), db, &self.env1, &self.env2).await?;

        Ok(())
    }
}
