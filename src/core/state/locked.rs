use std::fs;

use anyhow::Result;

use crate::core::crypto::decrypt;
use crate::core::crypto::header::EnvelopeFileHeader;
use crate::core::{envelope_path, envelope_tmp_path};

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

    /// Decrypts the envelope database file.
    ///
    /// Reads the encrypted file, decrypts it with the provided password,
    /// and writes the decrypted SQLite database back to disk using an
    /// atomic write (temp file + rename).
    ///
    /// Consumes self since the envelope is no longer locked after this
    /// operation.
    pub fn unlock(self, password: &str) -> Result<()> {
        let path = envelope_path()?;
        let plaintext = decrypt(&self.ciphertext, &self.header, password.as_bytes())?;

        // Atomic write: write to temp file, then rename
        let tmp_path = envelope_tmp_path()?;
        fs::write(&tmp_path, &plaintext)?;
        fs::rename(&tmp_path, &path)?;

        Ok(())
    }
}
