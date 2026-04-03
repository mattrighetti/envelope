use std::fs;

use anyhow::{Result, bail};
use zeroize::Zeroizing;

use crate::core::crypto::decrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::state::UnlockedEnvelope;
use crate::core::{envelope_path_exists, envelope_tmp_path};

/// Represents a locked (encrypted) envelope.
///
/// Contains the encrypted ciphertext and header information needed for
/// decryption.
#[derive(Debug)]
pub struct LockedEnvelope {
    header: EnvelopeFileHeader,
    ciphertext: Vec<u8>,
}

impl LockedEnvelope {
    pub(crate) fn new(header: EnvelopeFileHeader, ciphertext: Vec<u8>) -> Self {
        Self { header, ciphertext }
    }

    /// Persists the encrypted envelope to disk.
    pub fn store(self) -> Result<()> {
        let Some(path) = envelope_path_exists()? else {
            bail!("BUG: envelope file does not exist");
        };

        let tmp_path = envelope_tmp_path()?;
        let data = [
            self.header.to_bytes().as_slice(),
            self.ciphertext.as_slice(),
        ]
        .concat();

        fs::write(&tmp_path, &data)?;
        fs::rename(&tmp_path, &path)?;
        Ok(())
    }

    /// Decrypts the envelope database file.
    ///
    /// Reads the encrypted file, decrypts it with the provided password,
    /// and returns `UnlockedEnvelope` loaded in memory.
    ///
    /// Consumes self since the envelope is no longer locked after this
    /// operation.
    pub(crate) async fn unlock(self, password: &str) -> Result<UnlockedEnvelope> {
        let plaintext =
            decrypt(&self.ciphertext, &self.header, password.as_bytes()).map(Zeroizing::new)?;

        UnlockedEnvelope::open_in_memory(plaintext.as_slice()).await
    }
}
