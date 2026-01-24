mod locked;
mod unlocked;

pub use locked::LockedEnvelope;
pub use unlocked::UnlockedEnvelope;

use std::fs::File;
use std::io::{Read, Result};

use crate::core::crypto::header::{EnvelopeFileHeader, HEADER_SIZE, MAGIC_NUMBER};
use crate::core::envelope_path;
use crate::std_err;

/// sqlite magic number: <https://www.sqlite.org/fileformat.html>
const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";

/// Represents the detected state of the envelope file.
///
/// The envelope can be in one of two states:
/// - [`Locked`](Self::Locked): Encrypted with XChaCha20-Poly1305, requires password to access
/// - [`Unlocked`](Self::Unlocked): Plain SQLite database, ready for use
#[derive(Debug)]
pub enum EnvelopeState {
    /// Envelope is encrypted and requires a password to unlock.
    Locked(LockedEnvelope),
    /// Envelope is a plain SQLite database, ready for use.
    Unlocked,
}

/// Detects the current state of the envelope file.
///
/// This is a synchronous operation that only reads the file header.
/// Database connection happens separately via `UnlockedEnvelope::open()`.
pub fn detect() -> Result<Option<EnvelopeState>> {
    let path = envelope_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let mut file = File::open(&path)?;
    let mut buf = [0u8; HEADER_SIZE];
    let bytes_read = file.read(&mut buf)?;

    // check for encrypted envelope
    if bytes_read >= MAGIC_NUMBER.len() && buf.starts_with(MAGIC_NUMBER) {
        if bytes_read >= HEADER_SIZE
            && let Ok(header) = EnvelopeFileHeader::try_from(&buf[..])
        {
            // valid locked envelope file
            let mut ciphertext = Vec::new();
            file.read_to_end(&mut ciphertext)
                .map_err(|_| std_err!("error reading locked envelope"))?;

            return Ok(Some(EnvelopeState::Locked(LockedEnvelope::new(
                header, ciphertext,
            ))));
        }

        return Err(std_err!("invalid .envelope file"));
    }

    // check for sqlite database
    if bytes_read >= SQLITE_MAGIC.len() && buf.starts_with(SQLITE_MAGIC) {
        return Ok(Some(EnvelopeState::Unlocked));
    }

    Err(std_err!("invalid .envelope file"))
}
