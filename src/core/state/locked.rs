use std::fs;
use std::path::Path;

use anyhow::Result;
use zeroize::Zeroizing;

use crate::core::crypto::decrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::envelope_tmp_path_for;
use crate::core::state::UnlockedEnvelope;

/// Represents a locked (encrypted) envelope.
///
/// Contains the encrypted ciphertext and header information needed for
/// decryption.
#[derive(Debug)]
pub(crate) struct LockedEnvelope {
    header: EnvelopeFileHeader,
    ciphertext: Vec<u8>,
}

impl LockedEnvelope {
    pub(crate) fn new(header: EnvelopeFileHeader, ciphertext: Vec<u8>) -> Self {
        Self { header, ciphertext }
    }

    /// Persists the encrypted envelope to `path`.
    ///
    /// The target file is replaced atomically via a temporary file rename.
    pub(crate) fn store(self, path: &Path) -> Result<()> {
        let tmp_path = envelope_tmp_path_for(path);
        let data = [
            self.header.to_bytes().as_slice(),
            self.ciphertext.as_slice(),
        ]
        .concat();

        fs::write(&tmp_path, &data)?;
        fs::rename(&tmp_path, path)?;
        Ok(())
    }

    /// Decrypts the envelope database file.
    ///
    /// Reads the encrypted file, decrypts it with the provided password,
    /// and returns an [`UnlockedEnvelope`] loaded in memory.
    ///
    /// Decryption never restores a disk-backed unlocked value directly. The
    /// plaintext is materialized into an in-memory SQLite database first, and
    /// callers must explicitly persist it if they want a plain SQLite file on
    /// disk again.
    ///
    /// Consumes self since the envelope is no longer locked after this
    /// operation.
    pub(crate) async fn unlock(self, password: &str) -> Result<UnlockedEnvelope> {
        let plaintext =
            decrypt(&self.ciphertext, &self.header, password.as_bytes()).map(Zeroizing::new)?;

        UnlockedEnvelope::open_in_memory(plaintext.as_slice()).await
    }

    #[cfg(test)]
    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        [
            self.header.to_bytes().as_slice(),
            self.ciphertext.as_slice(),
        ]
        .concat()
    }
}
