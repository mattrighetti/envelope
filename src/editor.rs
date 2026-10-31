use std::env;
use std::io::{Read, Seek, Write};
use std::path::Path;

use anyhow::{Context, Result, ensure};
use tempfile::Builder as TempFileBuilder;

use crate::subproc::ChildProcess;

pub fn spawn_with(data: &[u8]) -> Result<Vec<u8>> {
    let editor = env::var("ENVELOPE_EDITOR")
        .or_else(|_| env::var("GIT_EDITOR"))
        .unwrap_or_else(|_| String::from("vi"));

    let current_directory = env::current_dir().context("failed to get current directory")?;
    spawn_with_editor(data, &editor, &current_directory)
}

fn spawn_with_editor(data: &[u8], editor: &str, current_directory: &Path) -> Result<Vec<u8>> {
    let mut file = TempFileBuilder::new()
        .prefix(".ENVELOPE_EDITMSG.")
        .tempfile_in(current_directory)
        .context("failed to create temporary edit file")?;

    let edit_path = file
        .path()
        .to_str()
        .context("current directory path contains invalid characters")?
        .to_owned();

    file.write_all(data)
        .context("failed to write temporary edit file")?;
    file.write_all(b"\n\n# Comment variables to remove them\n")
        .context("failed to write temporary edit file")?;
    file.flush()
        .context("failed to flush temporary edit file")?;

    let editor_args = &[edit_path.as_str()];
    let status = ChildProcess::new(&editor, editor_args, &[])
        .run()
        .context("failed to launch editor")?;
    ensure!(status.success(), "editor exited unsuccessfully: {status}");

    file.rewind()
        .context("failed to rewind temporary edit file")?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .context("failed to read temporary edit file")?;

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn nonzero_editor_status_is_an_error_and_cleans_up() {
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

        let remaining_edit_files = directory
            .path()
            .read_dir()
            .expect("the test should read the temporary directory")
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(".ENVELOPE_EDITMSG")
            })
            .count();
        assert_eq!(remaining_edit_files, 0);
    }
}
