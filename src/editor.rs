use std::{
    env::temp_dir,
    fs::{File, OpenOptions},
    io::{BufReader, Result, Write},
};

use crate::subproc::ChildProcess;
use crate::{other_err, other_str_err};

fn envelope_editor() -> String {
    let editor = "vi";

    if let Some(e) = std::env::var_os("ENVELOPE_EDITOR") {
        if let Some(e) = e.to_str() {
            return e.to_string();
        }
    }

    if let Some(e) = std::env::var_os("GIT_EDITOR") {
        if let Some(e) = e.to_str() {
            return e.to_string();
        }
    }

    editor.to_string()
}

fn writer() -> Result<File> {
    let pb = temp_dir().join("ENVELOPE_EDITMSG");

    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(true)
        .open(pb)
}

fn reader() -> Result<BufReader<File>> {
    let pb = temp_dir().join("ENVELOPE_EDITMSG");
    let file = OpenOptions::new().read(true).open(pb)?;

    Ok(BufReader::new(file))
}

fn prepare_file(data: &[u8]) -> Result<()> {
    let mut file = writer()?;
    file.write_all(data)?;
    file.write(b"\n\n")?;
    file.write(b"# Comment variables to remove them")?;

    Ok(())
}

pub fn spawn_with(data: &[u8]) -> Result<BufReader<File>> {
    prepare_file(data)?;

    {
        let editor = envelope_editor();
        if let Some(pb) = temp_dir().join("ENVELOPE_EDITMSG").to_str() {
            let cmd = ChildProcess::new(&editor, &[pb], &[]);
            cmd.run_shell_command()
                .map_err(|e| other_err!("error running child process", e))?;
        } else {
            return Err(other_str_err!("cannot run editor"));
        }
    }

    reader()
}
