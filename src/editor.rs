use std::env::{self};
use std::fs::File;
use std::io::{Read, Result};

use crate::std_err;
use crate::subproc::ChildProcess;

pub fn spawn_with(data: &[u8]) -> Result<Vec<u8>> {
    let editor = std::env::var("ENVELOPE_EDITOR")
        .or_else(|_| std::env::var("GIT_EDITOR"))
        .unwrap_or_else(|_| String::from("vi"));

    let Ok(curr_dir) = env::current_dir() else {
        return Err(std_err!("cannot get current dir"));
    };

    let pb = curr_dir.join(".ENVELOPE_EDITMSG");
    let Some(pb_str) = pb.to_str() else {
        return Err(std_err!("invalid path"));
    };

    std::fs::write(
        &pb,
        [data, b"\n\n# Comment variables to remove them\n"].concat(),
    )
    .map_err(|_| std_err!("failed to write edit message"))?;

    let args = &[pb_str];
    ChildProcess::new(&editor, args, &[])
        .run()
        .map_err(|e| std_err!("error running child process: {}", e))?;

    let mut file = File::open(&pb)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    std::fs::remove_file(pb)?;
    Ok(buf)
}
