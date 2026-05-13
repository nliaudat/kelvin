//! Encryption interface for the Kelvin cryptosystem.
//!
//! Encryption is performed using ChaCha20Poly1305 AEAD (or AES-256-GCM
//! when the `aes-ni` feature is enabled). The keystream is generated from
//! seeds extracted from the orbital simulation.
//!
//! ## Key Rotation
//!
//! When the current stream cipher approaches its maximum safe byte limit,
//! the next key is automatically derived from the key schedule. This ensures
//! forward secrecy and prevents nonce reuse.

use crate::{Kelvin, KelvinError};

impl Kelvin {
    /// Encrypt data in-place using AEAD.
    ///
    /// The buffer must have 16 extra bytes after the plaintext for the
    /// Poly1305/GMAC authentication tag.
    ///
    /// Automatically rotates the stream cipher key when the current key
    /// approaches its maximum safe byte limit.
    pub fn encrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        // Check if we need to rotate the key before encrypting
        let plaintext_len = data.len().saturating_sub(16) as u64;
        if self.stream.position() + plaintext_len > self.stream.max_safe_bytes() {
            self.rotate_key()?;
        }

        self.stream.encrypt_in_place(data)?;
        // Count only plaintext bytes, not the 16-byte AEAD tag
        self.bytes_processed += plaintext_len;
        Ok(())
    }
}
