use std::io::Read;
use std::io::Write;

use zeroize::Zeroize;

pub(crate) const MAGIC_NUMBER_LEN: usize = 12;
pub(crate) const VERSION_LEN: usize = 1;
pub(crate) const SALT_SIZE: usize = 16;
pub(crate) const NONCE_SIZE: usize = 24;
pub(crate) const HEADER_SIZE: usize = MAGIC_NUMBER_LEN + VERSION_LEN + SALT_SIZE + NONCE_SIZE; // 53 bytes

// First 4 bytes are SHA256("envelope")[0..4] to reduce collision risk with other formats.
// Remaining 8 bytes spell "ENVELOPE" for readability in hex dumps.
pub(crate) const MAGIC_NUMBER: &[u8; MAGIC_NUMBER_LEN] = b"\x4c\x50\x3c\xa6ENVELOPE";
pub(crate) const CURRENT_VERSION: u8 = 1;

/// Header for encrypted envelope files
#[derive(Debug)]
pub(crate) struct EnvelopeFileHeader {
    pub magic_number: [u8; MAGIC_NUMBER_LEN],
    pub version: u8,
    pub argon_salt: [u8; SALT_SIZE],
    pub xchacha_nonce: [u8; NONCE_SIZE],
}

impl Default for EnvelopeFileHeader {
    fn default() -> Self {
        Self {
            magic_number: *MAGIC_NUMBER,
            version: CURRENT_VERSION,
            argon_salt: [0u8; SALT_SIZE],
            xchacha_nonce: [0u8; NONCE_SIZE],
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum FileHeaderError {
    #[error("wrong header size")]
    WrongHeaderSize,
    #[error("wrong magic number")]
    WrongMagicNumber,
    #[error("parsing error: {0}")]
    ParsingError(&'static str),
}

impl TryFrom<&[u8]> for EnvelopeFileHeader {
    type Error = FileHeaderError;

    fn try_from(buffer: &[u8]) -> Result<Self, Self::Error> {
        if buffer.len() != HEADER_SIZE {
            return Err(FileHeaderError::WrongHeaderSize);
        }

        let mut reader = buffer;

        let mut magic_number = [0u8; MAGIC_NUMBER.len()];
        reader
            .read_exact(&mut magic_number)
            .map_err(|_| FileHeaderError::ParsingError("magic"))?;

        if &magic_number != MAGIC_NUMBER {
            return Err(FileHeaderError::WrongMagicNumber);
        }

        let mut version = [0u8; 1];
        reader
            .read_exact(&mut version)
            .map_err(|_| FileHeaderError::ParsingError("version"))?;

        let mut argon_salt = [0u8; SALT_SIZE];
        reader
            .read_exact(&mut argon_salt)
            .map_err(|_| FileHeaderError::ParsingError("argon_salt"))?;

        let mut xchacha_nonce = [0u8; NONCE_SIZE];
        reader
            .read_exact(&mut xchacha_nonce)
            .map_err(|_| FileHeaderError::ParsingError("xchacha_nonce"))?;

        Ok(EnvelopeFileHeader {
            magic_number,
            version: version[0],
            argon_salt,
            xchacha_nonce,
        })
    }
}

impl From<&EnvelopeFileHeader> for [u8; HEADER_SIZE] {
    fn from(header: &EnvelopeFileHeader) -> [u8; HEADER_SIZE] {
        let mut buffer = [0u8; HEADER_SIZE];
        let mut writer = &mut buffer[..];

        writer.write_all(&header.magic_number).unwrap();
        writer.write_all(&[header.version]).unwrap();
        writer.write_all(&header.argon_salt).unwrap();
        writer.write_all(&header.xchacha_nonce).unwrap();

        buffer
    }
}

impl Drop for EnvelopeFileHeader {
    fn drop(&mut self) {
        self.argon_salt.zeroize();
        self.xchacha_nonce.zeroize();
    }
}

impl EnvelopeFileHeader {
    pub(crate) fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        <[u8; HEADER_SIZE]>::from(self)
    }

    /// Returns the associated data (magic + version) for AEAD binding.
    /// This ensures ciphertext is bound to the header version, preventing
    /// tampering with the version byte without detection.
    pub(crate) fn associated_data(&self) -> [u8; MAGIC_NUMBER_LEN + VERSION_LEN] {
        let mut aad = [0u8; MAGIC_NUMBER_LEN + VERSION_LEN];
        aad[..MAGIC_NUMBER_LEN].copy_from_slice(&self.magic_number);
        aad[MAGIC_NUMBER_LEN] = self.version;
        aad
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct EnvelopeFileHeaderBuilder {
        buffer: Vec<u8>,
    }

    impl EnvelopeFileHeaderBuilder {
        fn new() -> Self {
            Self {
                buffer: Vec::with_capacity(HEADER_SIZE),
            }
        }

        fn magic(mut self, magic: &[u8]) -> Self {
            self.buffer.extend_from_slice(magic);
            self
        }

        fn version(mut self, version: u8) -> Self {
            self.buffer.push(version);
            self
        }

        fn argon_salt(mut self, salt: &[u8]) -> Self {
            self.buffer.extend_from_slice(salt);
            self
        }

        fn xchacha_nonce(mut self, nonce: &[u8]) -> Self {
            self.buffer.extend_from_slice(nonce);
            self
        }

        fn build(self) -> [u8; HEADER_SIZE] {
            assert_eq!(self.buffer.len(), HEADER_SIZE, "Header size mismatch");
            let mut output = [0u8; HEADER_SIZE];
            output.copy_from_slice(&self.buffer);
            output
        }
    }

    fn create_valid_header_buffer() -> [u8; HEADER_SIZE] {
        EnvelopeFileHeaderBuilder::new()
            .magic(MAGIC_NUMBER)
            .version(CURRENT_VERSION)
            .argon_salt(&[0x01; SALT_SIZE])
            .xchacha_nonce(&[0x02; NONCE_SIZE])
            .build()
    }

    #[test]
    fn test_valid_header_parsing() {
        let buffer = create_valid_header_buffer();
        let header = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(header.is_ok());
        let header = header.unwrap();

        assert_eq!(header.magic_number, *MAGIC_NUMBER);
        assert_eq!(header.version, CURRENT_VERSION);
        assert_eq!(header.argon_salt, [0x01; SALT_SIZE]);
        assert_eq!(header.xchacha_nonce, [0x02; NONCE_SIZE]);
    }

    #[test]
    fn test_wrong_header_size_too_small() {
        let buffer = [0u8; 32]; // Only 32 bytes instead of 53
        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongHeaderSize
        ));
    }

    #[test]
    fn test_wrong_header_size_too_large() {
        let buffer = [0u8; 128]; // 128 bytes instead of 53
        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongHeaderSize
        ));
    }

    #[test]
    fn test_wrong_magic_number() {
        let mut buffer = create_valid_header_buffer();
        buffer[0..12].copy_from_slice(b"WRONG_MAGIC!");

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongMagicNumber
        ));
    }

    #[test]
    fn test_partially_wrong_magic_number() {
        let mut buffer = create_valid_header_buffer();
        buffer[11] = b'X'; // Change last byte of magic number

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongMagicNumber
        ));
    }

    #[test]
    fn test_empty_buffer() {
        let buffer: &[u8] = &[];
        let result = EnvelopeFileHeader::try_from(buffer);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongHeaderSize
        ));
    }

    #[test]
    fn test_all_zeros_except_magic() {
        let mut buffer = [0u8; HEADER_SIZE];
        buffer[0..12].copy_from_slice(MAGIC_NUMBER);

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.version, 0);
        assert_eq!(header.argon_salt, [0; SALT_SIZE]);
    }

    #[test]
    fn test_all_ones_except_magic() {
        let mut buffer = [0xFFu8; HEADER_SIZE];
        buffer[0..12].copy_from_slice(MAGIC_NUMBER);

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.version, 0xFF);
        assert_eq!(header.argon_salt, [0xFF; SALT_SIZE]);
    }

    #[test]
    fn test_boundary_values() {
        let mut buffer = create_valid_header_buffer();
        buffer[12] = u8::MAX; // version = 255

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.version, u8::MAX);
    }

    #[test]
    fn test_roundtrip_with_to_bytes() {
        let original_buffer = create_valid_header_buffer();
        let header = EnvelopeFileHeader::try_from(&original_buffer[..]).unwrap();
        let serialized = header.to_bytes();

        assert_eq!(original_buffer, serialized);
    }
}
