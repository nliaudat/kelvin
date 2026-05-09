//! Decryption interface for the Kelvin cryptosystem.
//!
//! Decryption is identical to encryption — XOR is its own inverse.
//! The same keystream is generated from the same orbital configuration.

use crate::{Kelvin, KelvinError};
use kelvin_stream::StreamCipher;

impl Kelvin {
    /// Decrypt data in-place.
    ///
    /// Identical to encryption — XOR with the ChaCha20 keystream.
    pub fn decrypt_in_place(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.stream.xor_in_place(data);
        self.bytes_processed += data.len() as u64;
        Ok(())
    }
}
