//! ChaCha20Poly1305 authenticated stream cipher wrapper.
//!
//! ## Security Notes
//!
//! ### Nonce Rotation
//!
//! The nonce is incremented after each `encrypt_in_place`/`decrypt_in_place`
//! call to prevent nonce reuse. This ensures that each message uses a unique
//! (key, nonce) pair, which is the fundamental AEAD security invariant.
//!
//! ### Invocation Limit
//!
//! ChaCha20Poly1305 (IETF variant) uses a 12-byte nonce with a 32-bit block
//! counter, allowing up to 2^32 - 1 blocks (≈256 GiB) per (key, nonce) pair.
//! Kelvin enforces a conservative **4 GiB plaintext limit** per key, with
//! automatic key rotation via the key schedule when the limit is approached.
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
use aead::{AeadCore, AeadInPlace, KeyInit};
use aead::generic_array::typenum::Unsigned;
use chacha20poly1305::ChaCha20Poly1305;

/// ChaCha20Poly1305 authenticated stream cipher wrapper.
///
/// Wraps the `chacha20poly1305` crate with:
/// - 32-byte key + 12-byte nonce (IETF variant)
/// - AEAD authentication (Poly1305 tag)
/// - Position tracking
/// - Safe byte limit enforcement
pub struct ChaChaStream {
    cipher: ChaCha20Poly1305,
    nonce: [u8; 12],
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
    /// Create a new ChaCha20Poly1305 authenticated stream cipher.
    ///
    /// `key` must be 32 bytes, `nonce` must be 12 bytes (IETF variant).
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        let cipher = ChaCha20Poly1305::new_from_slice(&key)
            .expect("ChaCha20Poly1305 key must be 32 bytes");

        // ChaCha20 IETF max: 2^32 - 1 blocks × 64 bytes ≈ 256 GiB
        // We use a conservative 4 GiB limit per key
        let max_bytes = 1 << 32;

        ChaChaStream {
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
        self.cipher = ChaCha20Poly1305::new_from_slice(&key)
            .expect("ChaCha20Poly1305 key must be 32 bytes");
        self.nonce = nonce;
        self.position = 0;
    }

    /// Check if the cipher has exceeded its safe byte limit.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.max_bytes
    }
}

impl StreamCipher for ChaChaStream {
    fn encrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error> {
        // AEAD encrypt_in_place_detached: buffer[..plaintext_len] is plaintext.
        // The tag is returned separately and appended at buffer[plaintext_len..].
        let tag_size = <ChaCha20Poly1305 as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let plaintext_len = buffer.len() - tag_size;
        let nonce = chacha20poly1305::Nonce::from_slice(&self.nonce);
        let (msg, tag_out) = buffer.split_at_mut(plaintext_len);
        let result = self.cipher.encrypt_in_place_detached(nonce, &[], msg);

        // Increment nonce regardless of success to prevent reuse and maintain sync
        for byte in self.nonce.iter_mut().rev() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }

        let tag = result?;
        tag_out.copy_from_slice(tag.as_slice());
        self.position += plaintext_len as u64;

        Ok(())
    }

    fn decrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error> {
        // AEAD decrypt_in_place_detached: buffer[..ciphertext_len] is ciphertext,
        // buffer[ciphertext_len..] contains the 16-byte tag.
        let tag_size = <ChaCha20Poly1305 as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let ciphertext_len = buffer.len() - tag_size;
        let nonce = chacha20poly1305::Nonce::from_slice(&self.nonce);
        let (msg, tag) = buffer.split_at_mut(ciphertext_len);
        let result = self.cipher.decrypt_in_place_detached(nonce, &[], msg, aead::Tag::<ChaCha20Poly1305>::from_slice(tag));

        // Increment nonce regardless of success to maintain sync
        for byte in self.nonce.iter_mut().rev() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }

        result?;
        self.position += ciphertext_len as u64;

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
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        let plaintext = b"Hello, Kelvin!";
        // Buffer needs space for plaintext + 16-byte tag
        let mut buffer = vec![0u8; plaintext.len() + 16];
        buffer[..plaintext.len()].copy_from_slice(plaintext);

        cipher.encrypt_in_place(&mut buffer).unwrap();
        // The first plaintext.len() bytes should now be ciphertext (different from plaintext)
        assert_ne!(&buffer[..plaintext.len()], plaintext);

        // Decrypt with new cipher at same position
        let mut cipher2 = ChaChaStream::new(test_key(), test_nonce());
        cipher2.decrypt_in_place(&mut buffer).unwrap();
        assert_eq!(&buffer[..plaintext.len()], plaintext);
    }

    #[test]
    fn test_aead_tag_verification() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        let plaintext = b"Hello, Kelvin!";
        let mut buffer = vec![0u8; plaintext.len() + 16];
        buffer[..plaintext.len()].copy_from_slice(plaintext);

        cipher.encrypt_in_place(&mut buffer).unwrap();

        // Corrupt the ciphertext
        buffer[0] ^= 0xFF;

        // Decryption should fail
        let mut cipher2 = ChaChaStream::new(test_key(), test_nonce());
        let result = cipher2.decrypt_in_place(&mut buffer);
        assert!(result.is_err(), "AEAD should detect tampered ciphertext");
    }

    #[test]
    fn test_aead_deterministic() {
        let mut c1 = ChaChaStream::new(test_key(), test_nonce());
        let mut c2 = ChaChaStream::new(test_key(), test_nonce());

        let mut buf1 = vec![0u8; 64 + 16];
        let mut buf2 = vec![0u8; 64 + 16];

        c1.encrypt_in_place(&mut buf1).unwrap();
        c2.encrypt_in_place(&mut buf2).unwrap();

        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_aead_different_keys() {
        let mut key2 = test_key();
        key2[0] ^= 0x01;

        let mut c1 = ChaChaStream::new(test_key(), test_nonce());
        let mut c2 = ChaChaStream::new(key2, test_nonce());

        let mut buf1 = vec![0u8; 64 + 16];
        let mut buf2 = vec![0u8; 64 + 16];

        c1.encrypt_in_place(&mut buf1).unwrap();
        c2.encrypt_in_place(&mut buf2).unwrap();

        assert_ne!(buf1, buf2);
    }

    #[test]
    fn test_position_tracking() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        assert_eq!(cipher.position(), 0);

        let mut buf = vec![0u8; 10 + 16];
        cipher.encrypt_in_place(&mut buf).unwrap();
        // Position counts plaintext bytes only (not the 16-byte tag)
        assert_eq!(cipher.position(), 10);

        let mut buf2 = vec![0u8; 20 + 16];
        cipher.encrypt_in_place(&mut buf2).unwrap();
        assert_eq!(cipher.position(), 30);
    }

    #[test]
    fn test_rekey() {
        let mut cipher = ChaChaStream::new(test_key(), test_nonce());
        let mut buf = vec![0u8; 10 + 16];
        cipher.encrypt_in_place(&mut buf).unwrap();
        // Position counts plaintext bytes only
        assert_eq!(cipher.position(), 10);

        cipher.rekey(test_key(), test_nonce());
        assert_eq!(cipher.position(), 0);
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
        let mut buf = vec![0u8; 16]; // Just tag space (0 bytes plaintext)
        cipher.encrypt_in_place(&mut buf).unwrap();
        // Position counts plaintext bytes only — 0 plaintext bytes
        assert_eq!(cipher.position(), 0);
    }

    #[test]
    fn test_multi_message_round_trip() {
        // Verify that nonce increment keeps encryptor and decryptor in sync
        // across multiple sequential messages.
        let mut enc = ChaChaStream::new(test_key(), test_nonce());
        let mut dec = ChaChaStream::new(test_key(), test_nonce());

        let msg1 = b"First message";
        let msg2 = b"Second message, longer!";

        // Encrypt msg1
        let mut buf1 = vec![0u8; msg1.len() + 16];
        buf1[..msg1.len()].copy_from_slice(msg1);
        enc.encrypt_in_place(&mut buf1).unwrap();

        // Encrypt msg2
        let mut buf2 = vec![0u8; msg2.len() + 16];
        buf2[..msg2.len()].copy_from_slice(msg2);
        enc.encrypt_in_place(&mut buf2).unwrap();

        // Decrypt msg1 (dec starts at same nonce as enc)
        dec.decrypt_in_place(&mut buf1).unwrap();
        assert_eq!(&buf1[..msg1.len()], msg1, "msg1 should round-trip");

        // Decrypt msg2 (nonce advanced by one)
        dec.decrypt_in_place(&mut buf2).unwrap();
        assert_eq!(&buf2[..msg2.len()], msg2, "msg2 should round-trip");
    }
}
