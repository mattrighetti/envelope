use std::process::{Command, ExitStatus};

use crate::std_err;

pub struct ChildProcess<'a> {
    command: &'a str,
    args: &'a [&'a str],
    env: &'a [(&'a str, &'a str)],
    isolated: bool,
}

impl<'a> ChildProcess<'a> {
    pub fn new(command: &'a str, args: &'a [&'a str], env: &'a [(&'a str, &'a str)]) -> Self {
        Self {
            command,
            args,
            env,
            isolated: false,
        }
    }

    pub fn isolated(mut self, yes: bool) -> Self {
        self.isolated = yes;
        self
    }

    pub fn run(&self) -> Result<ExitStatus, Box<dyn std::error::Error>> {
        let mut cmd = Command::new(self.command);

        if self.isolated {
            cmd.env_clear();
            for var in ["PATH", "Path"] {
                if let Some(value) = std::env::var_os(var) {
                    cmd.env(var, value);
                }
            }
        }

        cmd.args(self.args.iter().copied());
        cmd.envs(self.env.iter().copied());

        let mut child = cmd
            .spawn()
            .map_err(|e| std_err!("failed to spawn {}: {}", self.command, e))?;

        Ok(child.wait()?)
    }
}
