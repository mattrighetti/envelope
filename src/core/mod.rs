pub(crate) mod crypto;
pub mod state;

use std::env;
use std::io::Result;
use std::path::PathBuf;

use crate::core::state::UnlockedEnvelope;
use crate::db::EnvelopeDb;
use crate::std_err;

const ENVELOPE_FILENAME: &str = ".envelope";
const ENVELOPE_FILENAME_TMP: &str = ".envelope.tmp";

/// Returns the path to the .envelope file in the current directory
pub(crate) fn envelope_path() -> Result<PathBuf> {
    Ok(env::current_dir()?.join(ENVELOPE_FILENAME))
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
    let path = envelope_path()?;
    let db_path = path
        .to_str()
        .ok_or_else(|| std_err!("invalid path encoding"))?;

    let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}?mode=rwc", db_path))
        .await
        .map_err(|e| std_err!("failed to create database: {}", e))?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| std_err!("failed to run migrations: {}", e))?;

    Ok(UnlockedEnvelope::with_db(EnvelopeDb::with(pool)))
}
