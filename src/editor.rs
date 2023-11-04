use std::{
    env::temp_dir,
    fs::{File, OpenOptions},
    io::{BufReader, Error, ErrorKind, Result, Write},
};

use crate::subproc::ChildProcess;

#[derive(Debug)]
pub struct Editor;

impl Editor {
    pub fn spawn_with(data: &[u8]) -> Result<BufReader<File>> {
        let fp = temp_dir().join("ENVELOPE_EDITMSG");

        {
            let mut file = OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .truncate(true)
                .open(fp.clone())?;

            file.write_all(data)?;
            file.write(b"\n\n")?;
            file.write(b"# Comment variables to remove them")?;
        }

        let cmd = ChildProcess::new("nvim", &[&fp.to_str().unwrap()], &[]);
        match cmd.run_shell_command() {
            Ok(_) => {
                let file = OpenOptions::new().read(true).open(fp.clone())?;
                Ok(BufReader::new(file))
            }
            Err(e) => Err(Error::new(
                ErrorKind::Other,
                format!("error running child process: {}", e),
            )),
        }
    }
}
