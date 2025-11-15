use std::io::Read;
use std::io::Write;

use zeroize::Zeroize;

const HEADER_SIZE: usize = 45;
const MAGIC_NUMBER: &[u8; 12] = b"ENVELOPE_ENC";
const SALT_SIZE: usize = 16;
const NONCE_SIZE: usize = 16;

/// Header for encrypted envelope files
#[repr(C)]
#[derive(Debug)]
pub(crate) struct EnvelopeFileHeader {
    pub magic_number: [u8; MAGIC_NUMBER.len()],
    pub version: u8,
    pub salt: [u8; SALT_SIZE],
    pub nonce: [u8; NONCE_SIZE],
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
            .inspect_err(|e| println!("{:?}", e))
            .map_err(|_| FileHeaderError::ParsingError("magic"))?;

        if &magic_number != MAGIC_NUMBER {
            return Err(FileHeaderError::WrongMagicNumber);
        }

        let mut version = [0u8; 1];
        reader
            .read_exact(&mut version)
            .inspect_err(|e| println!("{:?}", e))
            .map_err(|_| FileHeaderError::ParsingError("version"))?;

        let mut salt = [0u8; SALT_SIZE];
        reader
            .read_exact(&mut salt)
            .inspect_err(|e| println!("{:?}", e))
            .map_err(|_| FileHeaderError::ParsingError("salt"))?;

        let mut nonce = [0u8; NONCE_SIZE];
        reader
            .read_exact(&mut nonce)
            .inspect_err(|e| println!("{:?}", e))
            .map_err(|_| FileHeaderError::ParsingError("nonce"))?;

        Ok(EnvelopeFileHeader {
            magic_number,
            version: version[0],
            salt,
            nonce,
        })
    }
}

impl From<&EnvelopeFileHeader> for [u8; HEADER_SIZE] {
    fn from(header: &EnvelopeFileHeader) -> [u8; HEADER_SIZE] {
        let mut buffer = [0u8; HEADER_SIZE];
        let mut writer = &mut buffer[..];

        writer.write_all(&header.magic_number).unwrap();
        writer.write_all(&[header.version]).unwrap();
        writer.write_all(&header.salt).unwrap();
        writer.write_all(&header.nonce).unwrap();

        buffer
    }
}

impl Drop for EnvelopeFileHeader {
    fn drop(&mut self) {
        self.salt.zeroize();
        self.nonce.zeroize();
    }
}

impl EnvelopeFileHeader {
    pub(crate) fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        <[u8; HEADER_SIZE]>::from(self)
    }
}

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

    fn salt(mut self, salt: &[u8]) -> Self {
        self.buffer.extend_from_slice(salt);
        self
    }

    fn nonce(mut self, nonce: &[u8]) -> Self {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_header_buffer() -> [u8; HEADER_SIZE] {
        EnvelopeFileHeaderBuilder::new()
            .magic(MAGIC_NUMBER)
            .version(1)
            .salt(&[0x01; SALT_SIZE])
            .nonce(&[0x02; NONCE_SIZE])
            .build()
    }

    #[test]
    fn test_valid_header_parsing() {
        let buffer = create_valid_header_buffer();
        let header = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(header.is_ok());
        let header = header.unwrap();

        assert_eq!(header.magic_number, *MAGIC_NUMBER);
        assert_eq!(header.version, 1);
        assert_eq!(header.salt, [0x01; SALT_SIZE]);
        assert_eq!(header.nonce, [0x02; NONCE_SIZE]);
    }

    #[test]
    fn test_wrong_header_size_too_small() {
        let buffer = [0u8; 32]; // Only 32 bytes instead of 64
        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            FileHeaderError::WrongHeaderSize
        ));
    }

    #[test]
    fn test_wrong_header_size_too_large() {
        let buffer = [0u8; 128]; // 128 bytes instead of 64
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
        assert_eq!(header.salt, [0; 16]);
    }

    #[test]
    fn test_all_ones_except_magic() {
        let mut buffer = [0xFFu8; HEADER_SIZE];
        buffer[0..12].copy_from_slice(MAGIC_NUMBER);

        let result = EnvelopeFileHeader::try_from(&buffer[..]);

        assert!(result.is_ok());
        let header = result.unwrap();
        assert_eq!(header.version, 0xFF);
        assert_eq!(header.salt, [0xFF; 16]);
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
