use std::fs::{self, File};
use std::io::Write;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteOwnedBuf, SqlitePoolOptions};

use super::LockedEnvelope;
use crate::core::crypto::encrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::{envelope_path, envelope_tmp_path};
use crate::db::EnvelopeDb;

/// Represents an unlocked (unencrypted) envelope with database access.
pub struct UnlockedEnvelope {
    pub db: EnvelopeDb,
}

impl UnlockedEnvelope {
    /// Creates an UnlockedEnvelope with an existing database connection.
    pub(crate) fn with_db(db: EnvelopeDb) -> Self {
        Self { db }
    }

    /// Opens an existing unlocked envelope database.
    ///
    /// This should only be called after `detect()` returns
    /// `DetectedEnvelope::Unlocked`.
    pub async fn open() -> Result<Self> {
        let path = envelope_path()?;
        let db_path = path
            .to_str()
            .context("current directory path contains invalid characters")?;

        let pool = sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{db_path}?mode=rw"))
            .await
            .context("failed to open .envelope database")?;

        Ok(Self::with_db(EnvelopeDb::with(pool)))
    }

    /// Opens an in-memory envelope from raw SQLite bytes.
    pub async fn open_in_memory(bytes: &[u8]) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .connect("sqlite::memory:")
            .await
            .context("failed to create in-memory database")?;

        let mut conn = pool
            .acquire()
            .await
            .context("failed to acquire in-memory connection")?;

        const RW: bool = false;
        let buf = SqliteOwnedBuf::try_from(bytes)
            .map_err(|e| anyhow::anyhow!("failed to prepare database buffer: {e}"))?;
        conn.deserialize(None, buf, RW)
            .await
            .context("failed to load database into memory")?;
        drop(conn);

        Ok(Self::with_db(EnvelopeDb::with(pool)))
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

        let plaintext = fs::read(&envelope_path()?)?;
        encrypt_and_write(&plaintext, password)
    }

    /// Serializes the in-memory database, encrypts it, and writes it back
    /// to disk atomically.
    pub async fn store_locked(self, password: &str) -> Result<LockedEnvelope> {
        let plaintext = self.db.serialize().await?;
        encrypt_and_write(&plaintext, password)
    }
}

fn encrypt_and_write(plaintext: &[u8], password: &str) -> Result<LockedEnvelope> {
    let path = envelope_path()?;
    let mut header = EnvelopeFileHeader::default();
    let ciphertext = encrypt(&mut header, plaintext, password.as_bytes())?;

    // Atomic write: write to temp file, then rename
    let tmp_path = envelope_tmp_path()?;
    let mut file = File::create(&tmp_path)?;
    file.write_all(&header.to_bytes())?;
    file.write_all(&ciphertext)?;
    file.sync_all()?;
    fs::rename(&tmp_path, &path)?;

    Ok(LockedEnvelope::new(header, ciphertext))
}
