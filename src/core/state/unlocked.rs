use std::fs;

use anyhow::{Context, Result, bail};
use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf, SqlitePoolOptions};

use super::LockedEnvelope;
use crate::core::crypto::encrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::{envelope_path_exists, envelope_tmp_path};
use crate::db::EnvelopeDb;

/// Represents an unlocked (unencrypted) envelope with database access.
pub struct UnlockedEnvelope {
    db: EnvelopeDb,
}

impl UnlockedEnvelope {
    /// Creates an UnlockedEnvelope with an existing database connection.
    pub(crate) fn with_db(db: EnvelopeDb) -> Self {
        Self { db }
    }

    pub fn db(&self) -> &EnvelopeDb {
        &self.db
    }

    /// Opens an existing unlocked envelope database at the given path.
    pub(super) async fn open_at(path: &std::path::Path) -> Result<Self> {
        let opts = SqliteConnectOptions::new()
            .filename(path)
            .create_if_missing(false);

        let pool = sqlx::sqlite::SqlitePool::connect_with(opts)
            .await
            .context("failed to open .envelope database")?;

        Ok(Self::with_db(EnvelopeDb::with(pool)))
    }

    /// Opens an in-memory envelope from raw SQLite bytes.
    pub(crate) async fn open_in_memory(bytes: &[u8]) -> Result<Self> {
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

    /// Encrypts the in-memory database with the provided password.
    ///
    /// Returns a `LockedEnvelope` without writing to disk. Use
    /// `LockedEnvelope::store()` on the result to persist it.
    ///
    /// Consumes self since the database is serialized and no longer needed.
    pub async fn lock(self, password: &str) -> Result<LockedEnvelope> {
        let plaintext = self.db.serialize().await?;
        let mut header = EnvelopeFileHeader::default();
        let ciphertext = encrypt(&mut header, &plaintext, password.as_bytes())?;
        Ok(LockedEnvelope::new(header, ciphertext))
    }

    /// Persists the unlocked database to disk as a plain SQLite file.
    pub async fn store(self) -> Result<()> {
        let Some(path) = envelope_path_exists()? else {
            bail!("BUG: envelope file does not exist");
        };

        let tmp_path = envelope_tmp_path()?;
        let data = self.db.serialize().await?;
        fs::write(&tmp_path, &data)?;
        fs::rename(&tmp_path, &path)?;
        Ok(())
    }
}
