//! Encryption interface for the Kelvin cryptosystem.
//!
//! Encryption is performed using ChaCha20Poly1305 AEAD (or AES-256-GCM
//! when the `aes-ni` feature is enabled). The keystream is generated from
//! seeds extracted from the orbital simulation.

use crate::{Kelvin, KelvinError};

impl Kelvin {
    /// Encrypt data in-place using AEAD.
    ///
    /// The buffer must have 16 extra bytes after the plaintext for the
    /// Poly1305/GMAC authentication tag.
    pub fn encrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.stream.encrypt_in_place(data)?;
        // Count only plaintext bytes, not the 16-byte AEAD tag
        self.bytes_processed += data.len().saturating_sub(16) as u64;
        Ok(())
    }
}
