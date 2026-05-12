//! AES-256-GCM authenticated stream cipher wrapper.
//!
//! Provides a hardware-accelerated authenticated alternative to ChaCha20Poly1305
//! when the `aes-ni` feature is enabled.
//!
//! ## References
//!
//! - NIST (2007). "Recommendation for Block Cipher Modes of Operation:
//!   Galois/Counter Mode (GCM) and GMAC." SP 800-38D.
//! - Dworkin, M. (2001). "Recommendation for Block Cipher Modes of
//!   Operation." NIST SP 800-38A.

use crate::traits::StreamCipher;
use aead::{AeadCore, AeadInPlace, KeyInit};
use aead::generic_array::typenum::Unsigned;
use aes_gcm::Aes256Gcm;

/// AES-256-GCM authenticated stream cipher wrapper.
///
/// Wraps the `aes-gcm` crate with:
/// - 32-byte key + 12-byte nonce (standard GCM IV)
/// - AEAD authentication (GMAC tag)
/// - Position tracking
/// - Safe byte limit enforcement
pub struct AesGcmStream {
    cipher: Aes256Gcm,
    nonce: [u8; 12],
    position: u64,
    max_bytes: u64,
}

impl core::fmt::Debug for AesGcmStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AesGcmStream")
            .field("position", &self.position)
            .field("max_bytes", &self.max_bytes)
            .finish()
    }
}

impl AesGcmStream {
    /// Create a new AES-256-GCM authenticated stream cipher.
    ///
    /// `key` must be 32 bytes, `nonce` must be 12 bytes (standard GCM IV).
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&key)
            .expect("AES-256-GCM key must be 32 bytes");

        // Conservative 4 GiB limit per key
        let max_bytes = 1 << 32;

        AesGcmStream {
            cipher,
            nonce,
            position: 0,
            max_bytes,
        }
    }

    /// Rekey the cipher with a new key and nonce.
    ///
    /// Resets the position counter.
    pub fn rekey(&mut self, key: [u8; 32], nonce: [u8; 12]) {
        self.cipher = Aes256Gcm::new_from_slice(&key)
            .expect("AES-256-GCM key must be 32 bytes");
        self.nonce = nonce;
        self.position = 0;
    }

    /// Check if the cipher has exceeded its safe byte limit.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.max_bytes
    }
}

impl StreamCipher for AesGcmStream {
    fn encrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error> {
        // AEAD encrypt_in_place_detached: buffer[..plaintext_len] is plaintext.
        // The tag is returned separately and appended at buffer[plaintext_len..].
        let tag_size = <Aes256Gcm as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let plaintext_len = buffer.len() - tag_size;
        let nonce = aes_gcm::Nonce::from_slice(&self.nonce);
        let (msg, tag_out) = buffer.split_at_mut(plaintext_len);
        let tag = self.cipher.encrypt_in_place_detached(nonce, &[], msg)?;
        tag_out.copy_from_slice(tag.as_slice());
        self.position += buffer.len() as u64;
        Ok(())
    }

    fn decrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error> {
        // AEAD decrypt_in_place_detached: buffer[..ciphertext_len] is ciphertext,
        // buffer[ciphertext_len..] contains the 16-byte tag.
        let tag_size = <Aes256Gcm as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let ciphertext_len = buffer.len() - tag_size;
        let nonce = aes_gcm::Nonce::from_slice(&self.nonce);
        let (msg, tag) = buffer.split_at_mut(ciphertext_len);
        self.cipher.decrypt_in_place_detached(nonce, &[], msg, aead::Tag::<Aes256Gcm>::from_slice(tag))?;
        self.position += buffer.len() as u64;
        Ok(())
    }

    fn position(&self) -> u64 {
        self.position
    }

    fn max_safe_bytes(&self) -> u64 {
        self.max_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for i in 0..32 {
            k[i] = i as u8;
        }
        k
    }

    fn test_nonce() -> [u8; 12] {
        let mut n = [0u8; 12];
        for i in 0..12 {
            n[i] = (i + 32) as u8;
        }
        n
    }

    #[test]
    fn test_aead_round_trip() {
        let mut cipher = AesGcmStream::new(test_key(), test_nonce());
        let plaintext = b"Hello, Kelvin!";
        let mut buffer = vec![0u8; plaintext.len() + 16];
        buffer[..plaintext.len()].copy_from_slice(plaintext);

        cipher.encrypt_in_place(&mut buffer).unwrap();
        assert_ne!(&buffer[..plaintext.len()], plaintext);

        let mut cipher2 = AesGcmStream::new(test_key(), test_nonce());
        cipher2.decrypt_in_place(&mut buffer).unwrap();
        assert_eq!(&buffer[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_aead_tag_verification() {
        let mut cipher = AesGcmStream::new(test_key(), test_nonce());
        let plaintext = b"Hello, Kelvin!";
        let mut buffer = vec![0u8; plaintext.len() + 16];
        buffer[..plaintext.len()].copy_from_slice(plaintext);

        cipher.encrypt_in_place(&mut buffer).unwrap();

        // Corrupt the ciphertext
        buffer[0] ^= 0xFF;

        let mut cipher2 = AesGcmStream::new(test_key(), test_nonce());
        let result = cipher2.decrypt_in_place(&mut buffer);
        assert!(result.is_err(), "AEAD should detect tampered ciphertext");
    }

    #[test]
    fn test_deterministic() {
        let mut c1 = AesGcmStream::new(test_key(), test_nonce());
        let mut c2 = AesGcmStream::new(test_key(), test_nonce());

        let mut buf1 = vec![0u8; 64 + 16];
        let mut buf2 = vec![0u8; 64 + 16];

        c1.encrypt_in_place(&mut buf1).unwrap();
        c2.encrypt_in_place(&mut buf2).unwrap();

        assert_eq!(buf1, buf2);
    }
}
