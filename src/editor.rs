use std::env::{self};
use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result};

use crate::subproc::ChildProcess;

pub fn spawn_with(data: &[u8]) -> Result<Vec<u8>> {
    let editor = std::env::var("ENVELOPE_EDITOR")
        .or_else(|_| std::env::var("GIT_EDITOR"))
        .unwrap_or_else(|_| String::from("vi"));

    let curr_dir = env::current_dir().context("failed to get current directory")?;

    let pb = curr_dir.join(".ENVELOPE_EDITMSG");
    let pb_str = pb
        .to_str()
        .context("current directory path contains invalid characters")?;

    std::fs::write(
        &pb,
        [data, b"\n\n# Comment variables to remove them\n"].concat(),
    )
    .context("failed to write temporary edit file")?;

    let args = &[pb_str];
    ChildProcess::new(&editor, args, &[])
        .run()
        .context("failed to launch editor")?;

    let mut file = File::open(&pb)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    std::fs::remove_file(pb)?;
    Ok(buf)
}
