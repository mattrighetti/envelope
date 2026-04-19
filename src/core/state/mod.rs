mod locked;
mod unlocked;

use std::fs::File;
use std::io::Read;

use anyhow::{Context, Result, bail};
pub(crate) use locked::LockedEnvelope;
pub(crate) use unlocked::UnlockedEnvelope;

use crate::core::crypto::header::{EnvelopeFileHeader, HEADER_SIZE, MAGIC_NUMBER};
use crate::core::envelope_path_exists;

/// sqlite magic number: <https://www.sqlite.org/fileformat.html>
const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Represents the detected state of the envelope file.
///
/// The envelope can be in one of two states:
/// - [`Locked`](Self::Locked): Encrypted with XChaCha20-Poly1305, requires
///   password to access
/// - [`Unlocked`](Self::Unlocked): Plain SQLite database, ready for use
#[derive(Debug)]
pub(crate) enum EnvelopeState {
    /// Envelope is encrypted and requires a password to unlock.
    Locked(LockedEnvelope),
    /// Envelope is a plain SQLite database, ready for use.
    Unlocked(UnlockedEnvelope),
}

/// Detects the current `.envelope` file, if present.
///
/// Returns:
/// - `Ok(None)` when no envelope file exists at the default location
/// - `Ok(Some(EnvelopeState::Unlocked(_)))` for a plain SQLite database
/// - `Ok(Some(EnvelopeState::Locked(_)))` for an encrypted envelope file
pub(crate) async fn detect() -> Result<Option<EnvelopeState>> {
    let Some(path) = envelope_path_exists()? else {
        return Ok(None);
    };

    detect_at(&path).await.map(Some)
}

/// Detects the envelope state from a specific file path.
///
/// This is the path-based implementation behind [`detect`] and is kept
/// separate so tests can exercise format detection without mutating process
/// state.
async fn detect_at(path: &std::path::Path) -> Result<EnvelopeState> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; HEADER_SIZE];
    let bytes_read = file.read(&mut buf)?;

    // check for sqlite database
    if bytes_read >= SQLITE_MAGIC.len() && buf[..SQLITE_MAGIC.len()] == *SQLITE_MAGIC {
        let envelope = UnlockedEnvelope::open_at(path).await?;
        return Ok(EnvelopeState::Unlocked(envelope));
    }

    // check for encrypted envelope
    if bytes_read >= MAGIC_NUMBER.len() && buf.starts_with(MAGIC_NUMBER) {
        file.read_exact(&mut buf[bytes_read..])
            .context("corrupted .envelope file: header is truncated")?;

        let header = EnvelopeFileHeader::try_from(&buf[..])
            .context("corrupted .envelope file: header is malformed")?;

        // valid locked envelope file
        let mut ciphertext = Vec::new();
        file.read_to_end(&mut ciphertext)
            .context("failed to read encrypted envelope data")?;

        return Ok(EnvelopeState::Locked(LockedEnvelope::new(
            header, ciphertext,
        )));
    }

    bail!("unrecognized .envelope file format")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use sqlx::SqlitePool;
    use tempfile::TempDir;

    use super::*;
    use crate::core::crypto::encrypt;
    use crate::core::crypto::header::EnvelopeFileHeader;
    use crate::db::EnvelopeDb;

    // -- state transition tests (all in-memory, no disk) --

    #[sqlx::test]
    async fn test_lock_unlock_roundtrip(pool: SqlitePool) {
        let envelope = UnlockedEnvelope::from_db(EnvelopeDb::with(pool));
        envelope
            .db()
            .insert("prod", "API_KEY", "secret123")
            .await
            .unwrap();

        let locked = envelope.lock("password").await.unwrap();
        let restored = locked.unlock("password").await.unwrap();

        let rows = restored.db().list_kv_in_env("prod").await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key, "API_KEY");
        assert_eq!(rows[0].value, "secret123");
    }

    #[sqlx::test]
    async fn test_lock_unlock_preserves_multiple_envs(pool: SqlitePool) {
        let envelope = UnlockedEnvelope::from_db(EnvelopeDb::with(pool));
        envelope
            .db()
            .insert("prod", "DB_HOST", "prod.db")
            .await
            .unwrap();
        envelope
            .db()
            .insert("staging", "DB_HOST", "staging.db")
            .await
            .unwrap();

        let locked = envelope.lock("pw").await.unwrap();
        let restored = locked.unlock("pw").await.unwrap();

        let all = restored.db().get_active_kv_in_env().await.unwrap();
        assert_eq!(all.len(), 2);
    }

    #[sqlx::test]
    async fn test_unlock_wrong_password(pool: SqlitePool) {
        let envelope = UnlockedEnvelope::from_db(EnvelopeDb::with(pool));
        let locked = envelope.lock("correct").await.unwrap();

        let err = locked
            .unlock("wrong")
            .await
            .expect_err("unlock with wrong password should fail");
        assert!(
            err.to_string().contains("decryption failed"),
            "error should mention decryption failure, got: {err}"
        );
    }

    #[sqlx::test]
    async fn test_lock_produces_different_ciphertext(pool: SqlitePool) {
        let e1 = UnlockedEnvelope::from_db(EnvelopeDb::with(pool));
        e1.db().insert("env", "K", "V").await.unwrap();
        let bytes = e1.db().serialize().await.unwrap();

        let e2 = UnlockedEnvelope::open_in_memory(&bytes).await.unwrap();

        let locked1 = e1.lock("pw").await.unwrap();
        let locked2 = e2.lock("pw").await.unwrap();

        assert_ne!(locked1.to_bytes(), locked2.to_bytes());
    }

    #[tokio::test]
    async fn test_open_in_memory_with_invalid_bytes() {
        // SqliteOwnedBuf accepts arbitrary bytes, but the resulting database
        // is corrupt and queries will fail.
        let envelope = UnlockedEnvelope::open_in_memory(b"not a sqlite database")
            .await
            .unwrap();
        let err = envelope
            .db()
            .get_active_kv_in_env()
            .await
            .expect_err("querying a corrupt database should fail");
        assert!(
            err.to_string().to_lowercase().contains("failed"),
            "error should indicate a query failure, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_open_in_memory_with_empty_bytes() {
        let result = UnlockedEnvelope::open_in_memory(&[]).await;
        assert!(result.is_err(), "empty bytes should fail");
    }

    #[sqlx::test]
    async fn test_store_persists_to_path(pool: SqlitePool) {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");
        let envelope = UnlockedEnvelope::from_db(EnvelopeDb::with(pool));
        envelope
            .db()
            .insert("prod", "API_KEY", "secret123")
            .await
            .unwrap();

        envelope.store(&path).await.unwrap();

        let restored = UnlockedEnvelope::open_at(&path).await.unwrap();
        let rows = restored.db().list_kv_in_env("prod").await.unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].key, "API_KEY");
        assert_eq!(rows[0].value, "secret123");
    }

    // -- detect_at() tests (use tempdir, no cwd mutation) --

    #[tokio::test]
    async fn test_detect_unlocked_sqlite() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");

        // Create a real SQLite database
        let pool =
            sqlx::sqlite::SqlitePool::connect(&format!("sqlite://{}?mode=rwc", path.display()))
                .await
                .unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        pool.close().await;

        let state = detect_at(&path).await.unwrap();
        assert!(
            matches!(state, EnvelopeState::Unlocked(_)),
            "SQLite file should be detected as Unlocked"
        );
    }

    #[tokio::test]
    async fn test_detect_locked_envelope() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");

        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"fake sqlite data for testing";
        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();

        let mut data = Vec::new();
        data.extend_from_slice(header.to_bytes().as_slice());
        data.extend_from_slice(&ciphertext);
        fs::write(&path, &data).unwrap();

        let state = detect_at(&path).await.unwrap();
        assert!(
            matches!(state, EnvelopeState::Locked(_)),
            "encrypted file should be detected as Locked"
        );
    }

    #[tokio::test]
    async fn test_detect_garbage_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");
        fs::write(&path, b"this is not sqlite nor an envelope").unwrap();

        let err = detect_at(&path)
            .await
            .expect_err("garbage file should fail detection");
        assert!(
            err.to_string().contains("unrecognized"),
            "error should say unrecognized format, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_detect_truncated_header() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".envelope");

        // Write the magic number + partial header (not enough for full HEADER_SIZE)
        let header = EnvelopeFileHeader::default();
        let full_header = header.to_bytes();
        // Magic is 12 bytes; write 20 bytes so it enters the encrypted branch
        // but read_exact for the remaining header bytes fails
        fs::write(&path, &full_header[..20]).unwrap();

        let err = detect_at(&path)
            .await
            .expect_err("truncated header should fail");
        assert!(
            err.to_string().contains("truncated") || err.to_string().contains("corrupted"),
            "error should indicate corruption, got: {err}"
        );
    }
}
