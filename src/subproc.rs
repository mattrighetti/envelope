use std::{
    collections::HashMap,
    io::Result,
    process::{Command, ExitStatus},
};

#[derive(Debug)]
pub struct ChildProcess {
    cmd: String,
    args: Vec<String>,
    envs: HashMap<String, String>,
}

impl ChildProcess {
    pub fn new(cmd: &str, args: &[&str], envs: &[(&str, &str)]) -> Self {
        let args: Vec<String> = args.into_iter().map(|x| x.to_string()).collect();
        let envs: HashMap<String, String> = envs
            .into_iter()
            .map(|x| (x.0.to_string(), x.1.to_string()))
            .collect();

        ChildProcess {
            cmd: cmd.into(),
            args,
            envs,
        }
    }

    pub fn run_shell_command(&self) -> Result<ExitStatus> {
        Command::new(&self.cmd)
            .args(&self.args)
            .envs(&self.envs)
            .spawn()?
            .wait()
    }
}
