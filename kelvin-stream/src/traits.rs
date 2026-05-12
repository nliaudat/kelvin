//! Stream cipher trait for the Kelvin cryptosystem.

/// Trait for authenticated stream ciphers used in the Kelvin cryptosystem.
pub trait StreamCipher: core::fmt::Debug {
    /// Encrypt data in-place using AEAD.
    ///
    /// `buffer[..plaintext_len]` contains plaintext.
    /// On success, `buffer[..plaintext_len]` contains ciphertext + appended tag.
    fn encrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error>;

    /// Decrypt data in-place using AEAD.
    ///
    /// `buffer[..ciphertext_len]` contains ciphertext + appended tag.
    /// On success, `buffer[..ciphertext_len - 16]` contains plaintext.
    fn decrypt_in_place(
        &mut self,
        buffer: &mut [u8],
    ) -> Result<(), aead::Error>;

    /// Get the current position in the keystream.
    fn position(&self) -> u64;

    /// Get the maximum safe bytes before rekeying.
    fn max_safe_bytes(&self) -> u64;
}
