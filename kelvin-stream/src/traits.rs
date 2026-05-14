//! Stream cipher trait for the Kelvin cryptosystem.

/// Trait for authenticated stream ciphers used in the Kelvin cryptosystem.
pub trait StreamCipher: core::fmt::Debug {
    /// Encrypt data in-place using AEAD.
    ///
    /// `buffer[..plaintext_len]` contains plaintext.
    /// On success, `buffer[..plaintext_len]` contains ciphertext + appended tag.
    fn encrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>;

    /// Decrypt data in-place using AEAD.
    ///
    /// `buffer` contains ciphertext + appended 16-byte tag.
    /// On success, `buffer[..buffer.len() - 16]` contains plaintext.
    fn decrypt_in_place(&mut self, buffer: &mut [u8]) -> Result<(), aead::Error>;

    /// Get the current position in the keystream.
    fn position(&self) -> u64;

    /// Get the maximum safe bytes before rekeying.
    fn max_safe_bytes(&self) -> u64;

    /// Rekey the cipher with a new key and nonce.
    ///
    /// Resets the position counter. Preserves the cipher variant
    /// (ChaCha20Poly1305 vs AES-256-GCM) chosen at construction time.
    fn rekey(&mut self, key: [u8; 32], nonce: [u8; 12]);

    /// Zeroize all key material in the cipher.
    ///
    /// Overwrites the internal key and nonce with zeros. After calling this,
    /// the cipher is no longer usable for encryption/decryption.
    fn zeroize_key_material(&mut self);
}
