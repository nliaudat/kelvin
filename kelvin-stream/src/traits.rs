//! Stream cipher trait for the Kelvin cryptosystem.

/// Trait for stream ciphers used in the Kelvin cryptosystem.
pub trait StreamCipher {
    /// XOR data in-place with the keystream.
    ///
    /// This is the core encryption/decryption operation.
    fn xor_in_place(&mut self, data: &mut [u8]);

    /// Get the current position in the keystream.
    fn position(&self) -> u64;

    /// Get the maximum safe bytes before rekeying.
    fn max_safe_bytes(&self) -> u64;
}
