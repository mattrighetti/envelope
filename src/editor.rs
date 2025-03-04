use std::env;
use std::fs::OpenOptions;
use std::io::{Read, Result, Write};

use crate::std_err;
use crate::subproc::ChildProcess;

fn editor_cmd() -> String {
    let editor = "vim";

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

pub fn spawn_with(data: &[u8]) -> Result<Vec<u8>> {
    let editor = editor_cmd();
    let pb = env::current_dir()?.join(".ENVELOPE_EDITMSG");

    {
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .truncate(true)
            .open(&pb)?;

        file.write_all(data)?;
        file.write_all(b"\n\n# Comment variables to remove them")?;
    }

    let args = &[pb.to_str().unwrap()];
    let cmd = ChildProcess::new(&editor, args, &[]);
    cmd.run_shell_command()
        .map_err(|e| std_err!("error running child process: {}", e))?;

    let mut file = OpenOptions::new().read(true).open(&pb)?;

    let mut buf = Vec::new();
    file.read_to_end(&mut buf).unwrap();

    std::fs::remove_file(pb)?;
    Ok(buf)
}
