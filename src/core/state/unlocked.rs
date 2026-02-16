use std::fs::{self, File};
use std::io::{Result, Write};

use super::LockedEnvelope;
use crate::core::crypto::encrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::{envelope_path, envelope_tmp_path};
use crate::db::EnvelopeDb;
use crate::std_err;

/// Represents an unlocked (unencrypted) envelope with database access.
pub struct UnlockedEnvelope {
    pub db: EnvelopeDb,
}

impl UnlockedEnvelope {
    /// Opens an existing unlocked envelope database.
    ///
    /// This should only be called after `detect()` returns
    /// `DetectedEnvelope::Unlocked`.
    pub async fn open() -> Result<Self> {
        let path = envelope_path()?;
        let db_path = path
            .to_str()
            .ok_or_else(|| std_err!("invalid path encoding"))?;

        let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}?mode=rw", db_path))
            .await
            .map_err(|e| std_err!("failed to open database: {}", e))?;

        Ok(Self {
            db: EnvelopeDb::with(pool),
        })
    }

    /// Creates an UnlockedEnvelope with an existing database connection.
    pub(crate) fn with_db(db: EnvelopeDb) -> Self {
        Self { db }
    }

    /// Encrypts the envelope database file.
    ///
    /// Reads the SQLite database, encrypts it with the provided password,
    /// and writes the encrypted file back to disk using an atomic write
    /// (temp file + rename).
    ///
    /// Consumes self since the database connection is no longer valid after
    /// the file is encrypted.
    pub fn lock(self, password: &str) -> Result<LockedEnvelope> {
        // Close the database connection before reading the file to ensure
        // all writes are flushed and we read a consistent state.
        drop(self.db);

        let path = envelope_path()?;
        let plaintext = fs::read(&path)?;

        let mut header = EnvelopeFileHeader::default();
        let ciphertext = encrypt(&mut header, &plaintext, password.as_bytes())?;

        // Atomic write: write to temp file, then rename
        let tmp_path = envelope_tmp_path()?;
        let mut file = File::create(&tmp_path)?;
        file.write_all(&header.to_bytes())?;
        file.write_all(&ciphertext)?;
        file.sync_all()?;
        fs::rename(&tmp_path, &path)?;

        Ok(LockedEnvelope::new(header, ciphertext))
    }
}
