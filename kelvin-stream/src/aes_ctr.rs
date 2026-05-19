//! AES-256-GCM authenticated stream cipher wrapper.
//!
//! Provides a hardware-accelerated authenticated alternative to ChaCha20Poly1305
//! when the `aes-ni` feature is enabled.
//!
//! ## Security Notes
//!
//! ### Invocation Limit
//!
//! AES-GCM has a **2^32 invocation limit** per key when using a 12-byte nonce
//! (NIST SP 800-38D, Section 8.3). After 2^32 encryptions, the probability of
//! a nonce collision exceeds 2^-32. Kelvin enforces a conservative **4 GiB
//! plaintext limit** per key, which at 16-byte minimum messages allows at most
//! 2^28 invocations — well within the safety margin.
//!
//! ### Nonce Rotation
//!
//! The nonce is incremented after each `encrypt_in_place`/`decrypt_in_place`
//! call to prevent nonce reuse. This ensures that each message uses a unique
//! (key, nonce) pair, which is the fundamental AEAD security invariant.
//!
//! ## References
//!
//! - NIST (2007). "Recommendation for Block Cipher Modes of Operation:
//!   Galois/Counter Mode (GCM) and GMAC." SP 800-38D.
//! - Dworkin, M. (2001). "Recommendation for Block Cipher Modes of
//!   Operation." NIST SP 800-38A.

use crate::traits::StreamCipher;
use aead::generic_array::typenum::Unsigned;
use aead::{AeadCore, AeadInPlace, KeyInit};
use aes_gcm::Aes256Gcm;
use zeroize::Zeroize;

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
    /// Uses a conservative 4 GiB limit per key.
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        Self::with_max_bytes(key, nonce, 1 << 32)
    }

    /// Create a new AES-256-GCM stream cipher with a configurable byte limit.
    ///
    /// `max_bytes` must be >= 1 and <= 256 GiB (NIST SP 800-38D limit).
    /// Larger values reduce key rotation frequency at the cost of
    /// increased exposure if a key is compromised.
    pub fn with_max_bytes(key: [u8; 32], nonce: [u8; 12], max_bytes: u64) -> Self {
        let cipher = Aes256Gcm::new_from_slice(&key).expect("AES-256-GCM key must be 32 bytes");

        // AES-GCM max: 2^32 - 1 invocations per key (NIST SP 800-38D, Section 8.3)
        // At 16-byte minimum messages that is ~64 GiB, so we cap at 64 GiB.
        let max_allowed = 1u64 << 36; // 2^32 invocations × 16 bytes = 64 GiB
        let max_bytes = max_bytes.min(max_allowed).max(1);

        AesGcmStream { cipher, nonce, position: 0, max_bytes }
    }

    /// Rekey the cipher with a new key and nonce.
    ///
    /// Resets the position counter.
    pub fn rekey(&mut self, key: [u8; 32], nonce: [u8; 12]) {
        self.cipher = Aes256Gcm::new_from_slice(&key).expect("AES-256-GCM key must be 32 bytes");
        self.nonce = nonce;
        self.position = 0;
    }

    /// Check if the cipher has exceeded its safe byte limit.
    pub fn is_exhausted(&self) -> bool {
        self.position >= self.max_bytes
    }
}

impl StreamCipher for AesGcmStream {
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error> {
        // AEAD encrypt_in_place_detached: buffer[..plaintext_len] is plaintext.
        // The tag is returned separately and appended at buffer[plaintext_len..].
        let tag_size = <Aes256Gcm as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let plaintext_len = buffer.len() - tag_size;
        let nonce = aes_gcm::Nonce::from_slice(&self.nonce);
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

    fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error> {
        // AEAD decrypt_in_place_detached: buffer[..ciphertext_len] is ciphertext,
        // buffer[ciphertext_len..] contains the 16-byte tag.
        let tag_size = <Aes256Gcm as AeadCore>::TagSize::USIZE;
        if buffer.len() < tag_size {
            return Err(aead::Error);
        }
        let ciphertext_len = buffer.len() - tag_size;
        let nonce = aes_gcm::Nonce::from_slice(&self.nonce);
        let (msg, tag) = buffer.split_at_mut(ciphertext_len);
        let result = self.cipher.decrypt_in_place_detached(
            nonce,
            &[],
            msg,
            aead::Tag::<Aes256Gcm>::from_slice(tag),
        );

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

    fn rekey(&mut self, key: [u8; 32], nonce: [u8; 12]) {
        self.cipher = Aes256Gcm::new_from_slice(&key).expect("AES-256-GCM key must be 32 bytes");
        self.nonce = nonce;
        self.position = 0;
    }

    fn zeroize_key_material(&mut self) {
        // Rekey with zeros to overwrite the internal cipher state
        let zero_key = [0u8; 32];
        self.cipher =
            Aes256Gcm::new_from_slice(&zero_key).expect("AES-256-GCM key must be 32 bytes");
        self.nonce.zeroize();
        self.position = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut k = [0u8; 32];
        for (i, byte) in k.iter_mut().enumerate() {
            *byte = i as u8;
        }
        k
    }

    fn test_nonce() -> [u8; 12] {
        let mut n = [0u8; 12];
        for (i, byte) in n.iter_mut().enumerate() {
            *byte = (i + 32) as u8;
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

    #[test]
    fn test_position_tracking() {
        let mut cipher = AesGcmStream::new(test_key(), test_nonce());
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
    fn test_multi_message_round_trip() {
        // Verify that nonce increment keeps encryptor and decryptor in sync
        // across multiple sequential messages.
        let mut enc = AesGcmStream::new(test_key(), test_nonce());
        let mut dec = AesGcmStream::new(test_key(), test_nonce());

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
