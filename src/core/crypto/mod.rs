use std::io::Result;

use argon2::{Argon2, Params};
use chacha20poly1305::XChaCha20Poly1305;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use header::EnvelopeFileHeader;
use rand::Rng;
use zeroize::Zeroizing;

use crate::std_err;

pub(crate) mod header;

// Argon2id parameters for key derivation.
//
// References:
// - OWASP Password Storage Cheat Sheet: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
// - RFC 9106 (Argon2): https://www.rfc-editor.org/rfc/rfc9106.html
const KIB: u32 = 1 << 10;
const KEY_LEN: usize = 32; // 256-bit key for XChaCha20-Poly1305

#[cfg(not(test))]
const ARGON_MEMORY_KIB: u32 = 256; // 256 KiB memory cost
#[cfg(not(test))]
const ARGON_TIME_COST: u32 = 3; // 3 iterations
#[cfg(not(test))]
const ARGON_PARALLELISM: u32 = 8; // 8 parallel threads

// Use minimal Argon2 params in tests for speed
#[cfg(test)]
const ARGON_MEMORY_KIB: u32 = 8; // 8 KiB
#[cfg(test)]
const ARGON_TIME_COST: u32 = 1; // 1 iteration
#[cfg(test)]
const ARGON_PARALLELISM: u32 = 1; // 1 thread

fn derive_key(password: &[u8], salt: &[u8]) -> Result<Zeroizing<Vec<u8>>> {
    let params = Params::new(
        ARGON_MEMORY_KIB * KIB,
        ARGON_TIME_COST,
        ARGON_PARALLELISM,
        Some(KEY_LEN),
    )
    .map_err(|e| std_err!("key derivation failed: {}", e))?;
    let argon2 = Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params);

    let mut key_bytes = Zeroizing::new(vec![0u8; KEY_LEN]);
    argon2
        .hash_password_into(password, salt, &mut key_bytes)
        .map_err(|e| std_err!("key derivation failed: {}", e))?;

    Ok(key_bytes)
}

pub(crate) fn encrypt(
    header: &mut EnvelopeFileHeader,
    blob: &[u8],
    password: &[u8],
) -> Result<Vec<u8>> {
    rand::rng().fill_bytes(&mut header.argon_salt);
    rand::rng().fill_bytes(&mut header.xchacha_nonce);

    let key = derive_key(password, &header.argon_salt)?;
    let aead = XChaCha20Poly1305::new(key.as_slice().into());
    let aad = header.associated_data();
    let ciphertext = aead
        .encrypt(
            header.xchacha_nonce.as_ref().into(),
            Payload {
                msg: blob,
                aad: &aad,
            },
        )
        .map_err(|e| std_err!("encryption failed: {}", e))?;

    Ok(ciphertext)
}

pub(crate) fn decrypt(
    blob: &[u8],
    header: &EnvelopeFileHeader,
    password: &[u8],
) -> Result<Vec<u8>> {
    if header.version != header::CURRENT_VERSION {
        return Err(std_err!(
            "unsupported envelope version: {} (expected {})",
            header.version,
            header::CURRENT_VERSION
        ));
    }

    let key = derive_key(password, &header.argon_salt)?;
    let aead = XChaCha20Poly1305::new(key.as_slice().into());
    let aad = header.associated_data();
    let decrypted = aead
        .decrypt(
            header.xchacha_nonce.as_ref().into(),
            Payload {
                msg: blob,
                aad: &aad,
            },
        )
        .map_err(|_| std_err!("decryption failed - wrong password?"))?;

    Ok(decrypted)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key_deterministic() {
        let password = b"password";
        let salt = [0x42; 16];

        let key1 = derive_key(password, &salt).unwrap();
        let key2 = derive_key(password, &salt).unwrap();

        assert_eq!(
            key1.as_slice(),
            key2.as_slice(),
            "same inputs should produce same key"
        );
    }

    #[test]
    fn test_derive_key_different_passwords() {
        let salt = [0x42; 16];

        let key1 = derive_key(b"password1", &salt).unwrap();
        let key2 = derive_key(b"password2", &salt).unwrap();

        assert_ne!(
            key1.as_slice(),
            key2.as_slice(),
            "different passwords should produce different keys"
        );
    }

    #[test]
    fn test_derive_key_different_salts() {
        let password = b"password";

        let key1 = derive_key(password, &[0x42; 16]).unwrap();
        let key2 = derive_key(password, &[0x99; 16]).unwrap();

        assert_ne!(
            key1.as_slice(),
            key2.as_slice(),
            "different salts should produce different keys"
        );
    }

    #[test]
    fn test_derive_key_length() {
        let key = derive_key(b"password", &[0x42; 16]).unwrap();
        assert_eq!(key.len(), KEY_LEN, "key should be correct length");
    }

    #[test]
    fn test_derive_key_empty_password() {
        let key = derive_key(b"", &[0x42; 16]).unwrap();
        assert_eq!(key.len(), KEY_LEN, "should handle empty password");
    }

    #[test]
    fn test_encrypt_succeeds() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";
        let password = b"password";

        let result = encrypt(&mut header, plaintext, password);
        assert!(result.is_ok(), "encryption should succeed");
        let ciphertext = encrypt(&mut header, plaintext, password).unwrap();
        assert_ne!(
            ciphertext.as_slice(),
            plaintext,
            "ciphertext should differ from plaintext"
        );
    }

    #[test]
    fn test_encrypt_populates_header() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";
        let password = b"password";

        let salt_before = header.argon_salt;
        let nonce_before = header.xchacha_nonce;

        encrypt(&mut header, plaintext, password).unwrap();

        assert_ne!(header.argon_salt, salt_before, "salt should be randomized");
        assert_ne!(
            header.xchacha_nonce, nonce_before,
            "nonce should be randomized"
        );
    }

    #[test]
    fn test_encrypt_nonce_randomness() {
        let plaintext = b"test data";
        let password = b"password";

        let mut header1 = EnvelopeFileHeader::default();
        let ciphertext1 = encrypt(&mut header1, plaintext, password).unwrap();

        let mut header2 = EnvelopeFileHeader::default();
        let ciphertext2 = encrypt(&mut header2, plaintext, password).unwrap();

        assert_ne!(
            ciphertext1, ciphertext2,
            "multiple encryptions should produce different outputs"
        );
        assert_ne!(
            header1.xchacha_nonce, header2.xchacha_nonce,
            "nonces should be different"
        );
    }

    #[test]
    fn test_encrypt_empty_data() {
        let mut header = EnvelopeFileHeader::default();
        let result = encrypt(&mut header, &[], b"password");

        assert!(result.is_ok(), "should encrypt empty data");
        assert!(
            !result.unwrap().is_empty(),
            "ciphertext should include auth tag"
        );
    }

    #[test]
    fn test_encrypt_unicode_password() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";
        let password = "pâsswörd🔐".as_bytes();

        let result = encrypt(&mut header, plaintext, password);
        assert!(result.is_ok(), "should handle unicode password");
    }

    #[test]
    fn test_decrypt_wrong_password() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        let result = decrypt(&ciphertext, &header, b"wrong");

        assert!(result.is_err(), "should fail with wrong password");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("decryption failed"),
            "should indicate decryption failure"
        );
    }

    #[test]
    fn test_decrypt_unsupported_version() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        header.version = 99;

        let result = decrypt(&ciphertext, &header, b"password");

        assert!(result.is_err(), "should fail with unsupported version");
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("unsupported envelope version"),
            "should mention version"
        );
        assert!(
            err_msg.contains("99"),
            "should show unsupported version number"
        );
    }

    #[test]
    fn test_decrypt_corrupted_ciphertext() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let mut ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        assert!(!ciphertext.is_empty());

        let mid = ciphertext.len() / 2;
        ciphertext[mid] ^= 0xFF;

        let result = decrypt(&ciphertext, &header, b"password");
        assert!(result.is_err(), "should fail with corrupted ciphertext");
    }

    #[test]
    fn test_decrypt_wrong_salt() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        header.argon_salt[0] ^= 0xFF;

        let result = decrypt(&ciphertext, &header, b"password");
        assert!(result.is_err(), "should fail with tampered salt");
    }

    #[test]
    fn test_decrypt_wrong_nonce() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        header.xchacha_nonce[0] ^= 0xFF;

        let result = decrypt(&ciphertext, &header, b"password");
        assert!(result.is_err(), "should fail with tampered nonce");
    }

    #[test]
    fn test_decrypt_tampered_magic_in_aad() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        header.magic_number[0] ^= 0xFF;

        let result = decrypt(&ciphertext, &header, b"password");
        assert!(
            result.is_err(),
            "should fail with tampered magic (AAD binding)"
        );
    }

    #[test]
    fn test_decrypt_empty_ciphertext() {
        let header = EnvelopeFileHeader::default();
        let result = decrypt(&[], &header, b"password");

        assert!(result.is_err(), "should fail with empty ciphertext");
    }

    #[test]
    fn test_decrypt_truncated_ciphertext() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";

        let mut ciphertext = encrypt(&mut header, plaintext, b"password").unwrap();
        ciphertext.truncate(5);

        let result = decrypt(&ciphertext, &header, b"password");
        assert!(result.is_err(), "should fail with truncated ciphertext");
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";
        let password = b"password";

        let ciphertext = encrypt(&mut header, plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "roundtrip should preserve data"
        );
    }

    #[test]
    fn test_roundtrip_empty_data() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = &[];
        let password = b"password";

        let ciphertext = encrypt(&mut header, plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(decrypted.as_slice(), plaintext, "should handle empty data");
    }

    #[test]
    fn test_roundtrip_binary_data() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext: Vec<u8> = (0u8..=255).collect();
        let password = b"password";

        let ciphertext = encrypt(&mut header, &plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(decrypted, plaintext, "should preserve all byte values");
    }

    #[test]
    fn test_roundtrip_large_data() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = vec![0x42; 1024 * 1024]; // 1 MB
        let password = b"password";

        let ciphertext = encrypt(&mut header, &plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(decrypted, plaintext, "should handle large data");
    }

    #[test]
    fn test_roundtrip_unicode_password() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = b"test data";
        let password = "pâsswörd🔐".as_bytes();

        let ciphertext = encrypt(&mut header, plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(
            decrypted.as_slice(),
            plaintext,
            "should handle unicode password"
        );
    }

    #[test]
    fn test_multiple_roundtrips() {
        let original = b"test data";
        let password = b"password";

        let mut header1 = EnvelopeFileHeader::default();
        let ciphertext1 = encrypt(&mut header1, original, password).unwrap();
        let decrypted1 = decrypt(&ciphertext1, &header1, password).unwrap();

        let mut header2 = EnvelopeFileHeader::default();
        let ciphertext2 = encrypt(&mut header2, &decrypted1, password).unwrap();
        let decrypted2 = decrypt(&ciphertext2, &header2, password).unwrap();

        assert_eq!(
            decrypted2.as_slice(),
            original,
            "multiple roundtrips should preserve data"
        );
    }

    #[test]
    fn test_roundtrip_single_byte() {
        let mut header = EnvelopeFileHeader::default();
        let plaintext = &[0x42u8];
        let password = b"password";

        let ciphertext = encrypt(&mut header, plaintext, password).unwrap();
        let decrypted = decrypt(&ciphertext, &header, password).unwrap();

        assert_eq!(decrypted.as_slice(), plaintext, "should handle single byte");
    }
}
