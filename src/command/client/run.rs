use anyhow::{Context, Result, ensure};
use clap::Parser;

use super::CommandOutcome;
use crate::db::EnvelopeDb;
use crate::subproc::ChildProcess;

#[derive(Parser)]
pub struct Cmd {
    /// Environment to use
    env: String,

    /// Do not inherit variables from the parent shell
    #[arg(short, long)]
    isolated: bool,

    /// Command and arguments to run
    #[arg(trailing_var_arg = true, allow_hyphen_values = true, required = true)]
    args: Vec<String>,
}

impl Cmd {
    pub async fn run(&self, db: &EnvelopeDb) -> Result<CommandOutcome> {
        ensure!(
            db.env_exists(&self.env).await?,
            "environment '{}' does not exist",
            self.env
        );

        let (command, remaining_args) = self.args.split_first().context("no command provided")?;

        let env_vars = db.list_kv_in_env(&self.env).await?;

        let env_refs: Vec<(&str, &str)> = env_vars
            .iter()
            .map(|row| (row.key.as_str(), row.value.as_str()))
            .collect();

        let args_refs: Vec<&str> = remaining_args.iter().map(String::as_str).collect();
        let status = ChildProcess::new(command, &args_refs, &env_refs)
            .isolated(self.isolated)
            .run()
            .context("failed to run command")?;

        if !status.success() {
            return Ok(CommandOutcome::ChildFailure { status });
        }

        Ok(CommandOutcome::Success)
    }
}

#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;

    use super::*;

    #[sqlx::test]
    async fn child_failure_returns_control_to_the_caller(pool: SqlitePool) {
        let db = EnvelopeDb::with(pool);
        db.insert("test", "KEY", "value").await.unwrap();
        let cmd = Cmd {
            env: "test".into(),
            isolated: false,
            args: vec!["sh".into(), "-c".into(), "exit 7".into()],
        };

        let result = cmd.run(&db).await;

        let outcome = result.expect("spawning the child should succeed");
        let CommandOutcome::ChildFailure { status } = outcome else {
            panic!("child failure should be returned as an outcome");
        };
        assert_eq!(status.code(), Some(7));
    }
}
