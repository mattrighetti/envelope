use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqliteOwnedBuf, SqlitePoolOptions};

use super::LockedEnvelope;
use crate::core::crypto::encrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::{envelope_path, envelope_tmp_path_for};
use crate::db::EnvelopeDb;

/// Represents an unlocked (unencrypted) envelope with database access.
///
/// The underlying database may be backed by a file on disk (when opened via
/// [`Self::init`] or [`Self::open_at`]) or by an in-memory SQLite instance
/// (when produced by decrypting a [`LockedEnvelope`]).
/// Either way, persistence is the caller's responsibility via [`Self::store`].
#[derive(Debug)]
pub(crate) struct UnlockedEnvelope {
    db: EnvelopeDb,
}

impl UnlockedEnvelope {
    #[cfg(test)]
    pub(super) fn from_db(db: EnvelopeDb) -> Self {
        Self { db }
    }

    /// Opens or creates the default `.envelope` SQLite database and runs
    /// pending migrations.
    pub(crate) async fn init() -> Result<Self> {
        let path = envelope_path()?;
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(&path)
                    .create_if_missing(true)
                    .foreign_keys(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .busy_timeout(std::time::Duration::from_secs(5)),
            )
            .await
            .context("failed to open .envelope database")?;

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .context("failed to initialize database schema")?;

        Ok(Self {
            db: EnvelopeDb::with(pool),
        })
    }

    pub(crate) fn db(&self) -> &EnvelopeDb {
        &self.db
    }

    /// Opens an existing unlocked envelope database at `path`.
    ///
    /// Unlike [`Self::init`], this does not create the file or run migrations.
    pub(super) async fn open_at(path: &Path) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(path)
                    .create_if_missing(false)
                    .foreign_keys(true)
                    .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                    .busy_timeout(std::time::Duration::from_secs(5)),
            )
            .await
            .context("failed to open .envelope database")?;

        Ok(Self {
            db: EnvelopeDb::with(pool),
        })
    }

    /// Opens an in-memory envelope from serialized SQLite bytes.
    ///
    /// The returned database is detached from disk and can be queried or
    /// re-encrypted like any other [`UnlockedEnvelope`].
    pub(crate) async fn open_in_memory(bytes: &[u8]) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .min_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .in_memory(true)
                    .foreign_keys(true)
                    .busy_timeout(std::time::Duration::from_secs(5)),
            )
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

        Ok(Self {
            db: EnvelopeDb::with(pool),
        })
    }

    /// Encrypts the current database contents with the provided password.
    ///
    /// Returns a [`LockedEnvelope`] without writing to disk. Use
    /// [`LockedEnvelope::store`] on the result to persist the encrypted bytes.
    ///
    /// Consumes self since the database is serialized and no longer needed.
    pub(crate) async fn lock(self, password: &str) -> Result<LockedEnvelope> {
        let plaintext = self.db.serialize().await?;
        let mut header = EnvelopeFileHeader::default();
        let ciphertext = encrypt(&mut header, &plaintext, password.as_bytes())?;
        Ok(LockedEnvelope::new(header, ciphertext))
    }

    /// Persists the unlocked database to `path` as a plain SQLite file.
    ///
    /// The target file is replaced atomically via a temporary file rename.
    pub(crate) async fn store(self, path: &Path) -> Result<()> {
        let tmp_path = envelope_tmp_path_for(path);
        let data = self.db.serialize().await?;
        fs::write(&tmp_path, &data)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }
}
