//! Decryption interface for the Kelvin cryptosystem.
//!
//! Decryption uses AEAD verification (ChaCha20Poly1305 or AES-256-GCM).
//! The same keystream is generated from the same orbital configuration.

use crate::{Kelvin, KelvinError};

impl Kelvin {
    /// Decrypt data in-place using AEAD.
    ///
    /// The buffer must contain ciphertext + 16-byte authentication tag.
    pub fn decrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.stream.decrypt_in_place(data)?;
        self.bytes_processed += data.len() as u64;
        Ok(())
    }
}
