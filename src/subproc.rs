use std::{
    io::Result,
    process::{Command, ExitStatus},
    collections::HashMap,
};

#[derive(Debug)]
pub struct ChildProcess<'a> {
    cmd: &'a str,
    args: &'a [&'a str],
    envs: HashMap<&'a str, &'a str>,
}

impl<'a> ChildProcess<'a> {
    pub fn new(cmd: &'a str, args: &'a [&'a str], envs: &'a [(&'a str, &'a str)]) -> Self {
        let envs = envs.iter().cloned().collect();
        ChildProcess {
            cmd: cmd.into(),
            args,
            envs,
        }
    }

    pub fn run_shell_command(&self) -> Result<ExitStatus> {
        Command::new(&self.cmd)
            .args(self.args)
            .envs(&self.envs)
            .spawn()?
            .wait()
    }
}
