pub(crate) mod crypto;
pub(crate) mod state;

use std::env;
use std::path::{Path, PathBuf};

use anyhow::Result;

const ENVELOPE_FILENAME: &str = ".envelope";
const ENVELOPE_FILENAME_TMP: &str = ".envelope.tmp";

/// Returns the path to the .envelope file in the current directory
pub(crate) fn envelope_path() -> Result<PathBuf> {
    Ok(env::current_dir()?.join(ENVELOPE_FILENAME))
}

/// Returns the path to the .envelope file only if it exists
pub(crate) fn envelope_path_exists() -> Result<Option<PathBuf>> {
    let path = envelope_path()?;
    Ok(path.exists().then_some(path))
}

/// Returns the temporary path used while atomically replacing `path`.
pub(crate) fn envelope_tmp_path_for(path: &Path) -> PathBuf {
    path.with_file_name(ENVELOPE_FILENAME_TMP)
}
