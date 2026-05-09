//! ChaCha20 stream cipher wrapper with rekeying support.
//!
//! ## References
//!
//! - Bernstein, D. J. (2008). "ChaCha, a Variant of Salsa20." *Workshop
//!   Record of SASC 2008: The State of the Art of Stream Ciphers*.
//!   — Original ChaCha20 specification.
//! - Nir, Y., & Langley, A. (2018). "ChaCha20 and Poly1305 for IETF
//!   Protocols." RFC 8439. doi:10.17487/RFC8439
//!   — ChaCha20 IETF standard with test vectors.
//! - Bernstein, D. J. (2008). "The Salsa20 Family of Stream Ciphers."
//!   *New Stream Cipher Designs*, 84–97. doi:10.1007/978-3-540-68351-3_6
//!   — Predecessor to ChaCha20, design rationale.

use crate::traits::StreamCipher;
use chacha20::{
    cipher::{KeyIvInit, StreamCipher as ChaCha20Trait, StreamCipherSeek},
    ChaCha20, Key, Nonce,
};

/// ChaCha20 stream cipher wrapper.
///
/// Wraps the `chacha20` crate with:
/// - 32-byte key + 12-byte nonce (IETF variant)
/// - Position tracking
/// - Safe byte limit enforcement
pub struct ChaChaStream {
    cipher: ChaCha20,
    position: u64,
    max_bytes: u64,
}

impl core::fmt::Debug for ChaChaStream {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ChaChaStream")
            .field("position", &self.position)
            .field("max_bytes", &self.max_bytes)
            .finish()
    }
}

impl ChaChaStream {
    /// Create a new ChaCha20 stream cipher.
    ///
    /// `key` must be 32 bytes, `nonce` must be 12 bytes (IETF variant).
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        let key = Key::from_slice(&key);
        let nonce = Nonce::from_slice(&nonce);
        let cipher = ChaCha20::new(key, nonce);

        // ChaCha20 IETF max: 2^32 - 1 blocks × 64 bytes ≈ 256 GiB
        // We use a conservative 4 GiB limit per key
        let max_bytes = 1 << 32;

        ChaChaStream {
            cipher,
            position: 0,
            max_bytes,
        }
    }

    /// Rekey the cipher with a new key and nonce.
    ///
    /// Resets the position counter.
    pub fn rekey(&mut self, key: [u8; 32], nonce: [u8; 12]) {
        let key = Key::from_slice(&key);
        let nonce = Nonce::from_slice(&nonce);
        self.cipher = ChaCha20::new(key, nonce);
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

impl StreamCipher for ChaChaStream {
    fn xor_in_place(&mut self, data: &mut [u8]) {
        self.cipher.apply_keystream(data.into());
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

    fn test_nonce() -> [u8; 12] {
        let mut n = [0u8; 12];
        for i in 0..12 {
            n[i] = (i + 32) as u8;
        }
        n
    }

    #[test]
    fn test_encrypt_decrypt() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        let mut data = b"Hello, Kelvin!".to_vec();
        let original = data.clone();

        cipher.xor_in_place(&mut data);
        assert_ne!(data, original);

        // Decrypt with new cipher at same position
        let mut cipher2 = ChaChaStream::new(test_key(), test_nonce());
        cipher2.xor_in_place(&mut data);
        assert_eq!(data, original);
    }

    #[test]
    fn test_position_tracking() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        assert_eq!(cipher.position(), 0);

        cipher.xor_in_place(&mut [0u8; 10]);
        assert_eq!(cipher.position(), 10);

        cipher.xor_in_place(&mut [0u8; 20]);
        assert_eq!(cipher.position(), 30);
    }

    #[test]
    fn test_rekey() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        cipher.xor_in_place(&mut [0u8; 10]);
        assert_eq!(cipher.position(), 10);

        cipher.rekey(test_key(), test_nonce());
        assert_eq!(cipher.position(), 0);
    }

    #[test]
    fn test_seek() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        cipher.seek(100);
        assert_eq!(cipher.position(), 100);
    }

    #[test]
    fn test_deterministic() {
        let mut c1 = ChaChaStream::new(test_key(), test_nonce());
        let mut c2 = ChaChaStream::new(test_key(), test_nonce());

        let mut d1 = [0u8; 64];
        let mut d2 = [0u8; 64];

        c1.xor_in_place(&mut d1);
        c2.xor_in_place(&mut d2);

        assert_eq!(d1, d2);
    }

    #[test]
    fn test_not_exhausted_initially() {
        let cipher = ChaChaStream::new(test_key(), test_nonce());
        assert!(!cipher.is_exhausted());
    }

    #[test]
    fn test_max_safe_bytes() {
        let cipher = ChaChaStream::new(test_key(), test_nonce());
        assert_eq!(cipher.max_safe_bytes(), 1 << 32);
    }

    #[test]
    fn test_empty_data() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        cipher.xor_in_place(&mut []);
        assert_eq!(cipher.position(), 0);
    }

    #[test]
    fn test_large_data() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        let mut data = vec![0xABu8; 10000];
        let original = data.clone();

        cipher.xor_in_place(&mut data);
        assert_ne!(data, original);

        let mut cipher2 = ChaChaStream::new(test_key(), test_nonce());
        cipher2.xor_in_place(&mut data);
        assert_eq!(data, original);
    }
}
