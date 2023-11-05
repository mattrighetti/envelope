use std::{
    env::temp_dir,
    fs::{File, OpenOptions},
    io::{BufReader, Result, Write},
    path::PathBuf,
};

use crate::other_err;
use crate::subproc::ChildProcess;

fn prepare_file(path: &PathBuf, data: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(path.clone())?;

    file.write_all(data)?;
    file.write(b"\n\n")?;
    file.write(b"# Comment variables to remove them")?;

    Ok(())
}

pub fn spawn_with(data: &[u8]) -> Result<BufReader<File>> {
    let fp = temp_dir().join("ENVELOPE_EDITMSG");
    prepare_file(&fp, data)?;

    let cmd = ChildProcess::new("nvim", &[&fp.to_str().unwrap()], &[]);
    cmd.run_shell_command()
        .map_err(|e| other_err!("error running child process", e))?;

    let file = OpenOptions::new().read(true).open(fp.clone())?;
    Ok(BufReader::new(file))
}
