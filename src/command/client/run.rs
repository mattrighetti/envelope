use std::io::Result;

use clap::Parser;

use crate::db::EnvelopeDb;
use crate::std_err;
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
    pub async fn run(&self, db: &EnvelopeDb) -> Result<()> {
        if !db.env_exists(&self.env).await? {
            return Err(std_err!("env {} does not exist", self.env));
        }

        let (command, remaining_args) = self
            .args
            .split_first()
            .ok_or_else(|| std_err!("no command provided"))?;

        let env_vars = db.list_kv_in_env(&self.env).await?;

        let env_refs: Vec<(&str, &str)> = env_vars
            .iter()
            .map(|row| (row.key.as_str(), row.value.as_str()))
            .collect();

        let args_refs: Vec<&str> = remaining_args.iter().map(String::as_str).collect();
        let status = ChildProcess::new(command, &args_refs, &env_refs)
            .isolated(self.isolated)
            .run()
            .map_err(|_| std_err!("encountered error while executing command."))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
