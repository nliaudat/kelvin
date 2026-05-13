//! Decryption interface for the Kelvin cryptosystem.
//!
//! Decryption uses AEAD verification (ChaCha20Poly1305 or AES-256-GCM).
//! The same keystream is generated from the same orbital configuration.
//!
//! ## Key Rotation
//!
//! When the current stream cipher approaches its maximum safe byte limit,
//! the next key is automatically derived from the key schedule. This ensures
//! forward secrecy and prevents nonce reuse.

use crate::{Kelvin, KelvinError};

impl Kelvin {
    /// Decrypt data in-place using AEAD.
    ///
    /// The buffer must contain ciphertext + 16-byte authentication tag.
    ///
    /// Automatically rotates the stream cipher key when the current key
    /// approaches its maximum safe byte limit.
    pub fn decrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        // Check if we need to rotate the key before decrypting
        let ciphertext_len = data.len().saturating_sub(16) as u64;
        if self.stream.position() + ciphertext_len > self.stream.max_safe_bytes() {
            self.rotate_key()?;
        }

        self.stream.decrypt_in_place(data)?;
        // Count only plaintext bytes, not the 16-byte AEAD tag
        self.bytes_processed += ciphertext_len;
        Ok(())
    }
}
