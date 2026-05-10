//! Encryption interface for the Kelvin cryptosystem.
//!
//! Encryption is performed by XORing plaintext with the ChaCha20 keystream.
//! The keystream is generated from seeds extracted from the orbital simulation.

use crate::{Kelvin, KelvinError};

impl Kelvin {
    /// Encrypt data in-place.
    ///
    /// This XORs the input data with the ChaCha20 keystream.
    /// Encryption and decryption are identical operations.
    pub fn encrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.stream.xor_in_place(data);
        self.bytes_processed += data.len() as u64;
        Ok(())
    }
}
