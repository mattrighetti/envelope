pub(crate) mod crypto;
pub mod state;

use std::env;
use std::path::PathBuf;

use anyhow::Result;

use crate::core::state::UnlockedEnvelope;

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

/// Returns the path to the temporary .envelope file
pub(crate) fn envelope_tmp_path() -> Result<PathBuf> {
    Ok(env::current_dir()?.join(ENVELOPE_FILENAME_TMP))
}

/// Initializes a new envelope database.
///
/// Creates the .envelope SQLite database file and runs migrations.
/// Consumes self and returns an UnlockedEnvelope on success.
pub async fn init() -> Result<UnlockedEnvelope> {
    UnlockedEnvelope::init().await
}
