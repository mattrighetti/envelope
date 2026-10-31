use std::env::{self};
use std::fs::File;
use std::io::Read;
use std::path::Path;

use anyhow::{Context, Result, ensure};

use crate::subproc::ChildProcess;

pub fn spawn_with(data: &[u8]) -> Result<Vec<u8>> {
    let editor = std::env::var("ENVELOPE_EDITOR")
        .or_else(|_| std::env::var("GIT_EDITOR"))
        .unwrap_or_else(|_| String::from("vi"));

    let curr_dir = env::current_dir().context("failed to get current directory")?;
    spawn_with_editor(data, &editor, &curr_dir)
}

fn spawn_with_editor(data: &[u8], editor: &str, directory: &Path) -> Result<Vec<u8>> {
    let pb = directory.join(".ENVELOPE_EDITMSG");
    let pb_str = pb
        .to_str()
        .context("current directory path contains invalid characters")?;

    std::fs::write(
        &pb,
        [data, b"\n\n# Comment variables to remove them\n"].concat(),
    )
    .context("failed to write temporary edit file")?;

    let editor_args = &[pb_str];
    let status = ChildProcess::new(&editor, editor_args, &[])
        .run()
        .context("failed to launch editor")?;
    ensure!(status.success(), "editor exited unsuccessfully: {status}");

    let mut file = File::open(&pb)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;

    std::fs::remove_file(pb)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn nonzero_editor_status_is_an_error() {
        let directory = TempDir::new().expect("the test should create a temporary directory");
        let editor = directory.path().join("failing-editor");
        fs::write(&editor, "#!/bin/sh\nexit 1\n").expect("the test should create an editor");

        let mut permissions = fs::metadata(&editor)
            .expect("the test should read editor permissions")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&editor, permissions)
            .expect("the test should make the editor executable");

        let error = spawn_with_editor(b"KEY=value", editor.to_str().unwrap(), directory.path())
            .expect_err("a failing editor should return an error");
        assert!(error.to_string().contains("editor exited unsuccessfully"));

    }
}
