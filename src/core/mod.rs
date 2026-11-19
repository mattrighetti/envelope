pub(crate) mod crypto;
pub(crate) mod state;

use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::Result;
use tempfile::Builder;

const ENVELOPE_FILENAME: &str = ".envelope";
const ENVELOPE_FILENAME_TMP_PREFIX: &str = ".envelope.tmp-";

/// Returns the path to the .envelope file in the current directory
pub(crate) fn envelope_path() -> Result<PathBuf> {
    Ok(env::current_dir()?.join(ENVELOPE_FILENAME))
}

/// Returns the path to the .envelope file only if it exists
pub(crate) fn envelope_path_exists() -> Result<Option<PathBuf>> {
    let path = envelope_path()?;
    Ok(path.exists().then_some(path))
}

/// Replaces an envelope via a temporary file, removing it on failure.
///
/// The temporary file is created alongside the destination so persisting it
/// is an atomic rename on the same filesystem.
pub(crate) fn store_bytes(path: &Path, data: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut file = Builder::new()
        .prefix(ENVELOPE_FILENAME_TMP_PREFIX)
        .tempfile_in(parent)?;
    file.write_all(data)?;
    file.persist(path).map_err(std::io::Error::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn store_replaces_destination_and_removes_temporary_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");
        fs::write(&path, b"original").unwrap();

        store_bytes(&path, b"replacement").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[test]
    fn failed_rename_removes_temporary_plaintext() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");
        fs::create_dir(&path).unwrap();
        fs::write(path.join("keep"), b"original").unwrap();

        let _error = store_bytes(&path, b"secret plaintext").unwrap_err();

        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
        assert_eq!(fs::read(path.join("keep")).unwrap(), b"original");
    }

    #[test]
    fn existing_legacy_temporary_file_is_preserved() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");
        let tmp_path = dir.path().join(".envelope.tmp");
        fs::write(&path, b"original").unwrap();
        fs::write(&tmp_path, b"existing").unwrap();

        store_bytes(&path, b"replacement").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"replacement");
        assert_eq!(fs::read(&tmp_path).unwrap(), b"existing");
        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
    }
}
