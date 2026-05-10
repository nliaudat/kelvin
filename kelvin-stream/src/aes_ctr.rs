//! AES-256-CTR stream cipher wrapper with rekeying support.
//!
//! Provides a hardware-accelerated alternative to ChaCha20 when the
//! `aes-ni` feature is enabled.

use crate::traits::StreamCipher;
use aes::Aes256;
use aes::cipher::{KeyIvInit, StreamCipher as StreamCipherTrait, StreamCipherSeek};
use ctr::Ctr128BE;

type Aes256Ctr = Ctr128BE<Aes256>;

/// AES-256-CTR stream cipher wrapper.
///
/// Wraps the `aes` and `ctr` crates with:
/// - 32-byte key + 16-byte IV
/// - Position tracking
/// - Safe byte limit enforcement
pub struct AesCtrStream {
    cipher: Aes256Ctr,
    position: u64,
    max_bytes: u64,
}

impl core::fmt::Debug for AesCtrStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AesCtrStream")
            .field("position", &self.position)
            .field("max_bytes", &self.max_bytes)
            .finish()
    }
}

impl AesCtrStream {
    /// Create a new AES-256-CTR stream cipher.
    ///
    /// `key` must be 32 bytes, `iv` must be 16 bytes.
    pub fn new(key: [u8; 32], iv: [u8; 16]) -> Self {
        let cipher = Aes256Ctr::new(&key.into(), &iv.into());

        // Max bytes limit. We use 4 GiB limit per key for consistency.
        let max_bytes = 1 << 32;

        AesCtrStream {
            cipher,
            position: 0,
            max_bytes,
        }
    }

    /// Rekey the cipher with a new key and IV.
    ///
    /// Resets the position counter.
    pub fn rekey(&mut self, key: [u8; 32], iv: [u8; 16]) {
        self.cipher = Aes256Ctr::new(&key.into(), &iv.into());
        self.position = 0;
    }

    /// Seek to a specific position in the keystream.
    pub fn seek(&mut self, position: u64) {
        self.cipher.seek(position);
        self.position = position;
    }

    /// Check if the cipher has exceeded its safe byte limit.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.max_bytes
    }
}

impl StreamCipher for AesCtrStream {
    fn xor_in_place(&mut self, data: &mut [u8]) {
        self.cipher.apply_keystream(data);
        self.position += data.len() as u64;
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

    fn test_iv() -> [u8; 16] {
        let mut n = [0u8; 16];
        for i in 0..16 {
            n[i] = (i + 32) as u8;
        }
        n
    }

    #[test]
    fn test_encrypt_decrypt() {
        let mut cipher = AesCtrStream::new(test_key(), test_iv());
        let mut data = b"Hello, Kelvin!".to_vec();
        let original = data.clone();

        cipher.xor_in_place(&mut data);
        assert_ne!(data, original);

        let mut cipher2 = AesCtrStream::new(test_key(), test_iv());
        cipher2.xor_in_place(&mut data);
        assert_eq!(data, original);
    }
}
