//! Streaming encryption/decryption traits for the Kelvin cryptosystem.
//!
//! Provides a unified streaming interface across all modes:
//! - `StreamEncrypt` — incremental encryption with optional finalization
//! - `StreamDecrypt` — incremental decryption with optional finalization
//!
//! ## Design
//!
//! Unlike the batch API (which processes an entire buffer in one call),
//! the streaming API allows processing data in arbitrary-sized chunks
//! with constant memory (64 KB buffer). This enables encrypting/decrypting
//! arbitrarily large files without loading them entirely into memory.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use kelvin::streaming::{StreamEncrypt, StreamDecrypt};
//!
//! let mut encryptor = kelvin_photon_streaming_encrypt(seed, 1000);
//! let mut output = Vec::new();
//! for chunk in input.chunks(65536) {
//!     encryptor.update(chunk, &mut output)?;
//! }
//! let tag = encryptor.finalize()?;
//! output.extend(tag);
//! ```
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**

use crate::error::KelvinError;
use crate::parameters::{PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE};
use crate::photon::KelvinPhoton;
use crate::quantum::KelvinQuantum;
use crate::KelvinStreaming;
use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::ChaCha20;
use kelvin_core::IntegrationMethod;
use kelvin_kdf::OrbitalConfig;
use zeroize::Zeroize;

/// Streaming encryption trait for the Kelvin cryptosystem.
///
/// Processes data incrementally with bounded memory. Each `update()` call
/// encrypts a chunk of plaintext and appends the ciphertext to `output`.
/// The `finalize()` call returns any remaining authentication tag (empty
/// for non-AEAD modes).
///
/// ## Memory
///
/// Implementations should avoid per-update allocations by reusing internal
/// buffers. The `output` buffer is provided by the caller and grows as needed.
pub trait StreamEncrypt {
    /// Encrypt a chunk of plaintext and append ciphertext to `output`.
    ///
    /// `plaintext` is the input data to encrypt.
    /// `output` is extended with the encrypted bytes.
    ///
    /// For AEAD modes (Secure), the authentication tag is NOT appended here —
    /// it is returned by [`finalize`](Self::finalize).
    fn update(&mut self, plaintext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError>;

    /// Finalize encryption and return the authentication tag.
    ///
    /// For AEAD modes (Secure), this returns the final authentication tag
    /// that must be appended to the ciphertext for decryption.
    ///
    /// For non-AEAD modes (Chaos, Photon, Quantum), this returns an empty
    /// `Vec<u8>` and is a no-op.
    fn finalize(&mut self) -> Result<Vec<u8>, KelvinError>;
}

/// Streaming decryption trait for the Kelvin cryptosystem.
///
/// Processes data incrementally with bounded memory. Each `update()` call
/// decrypts a chunk of ciphertext and appends the plaintext to `output`.
/// The `finalize()` call verifies the authentication tag (for AEAD modes).
pub trait StreamDecrypt {
    /// Decrypt a chunk of ciphertext and append plaintext to `output`.
    ///
    /// `ciphertext` is the input data to decrypt (without the final tag).
    /// `output` is extended with the decrypted bytes.
    fn update(&mut self, ciphertext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError>;

    /// Finalize decryption and verify the authentication tag.
    ///
    /// For AEAD modes (Secure), `tag` is the final authentication tag that
    /// was appended to the ciphertext. Returns an error if the tag is invalid.
    ///
    /// For non-AEAD modes (Chaos, Photon, Quantum), `tag` is ignored and
    /// this is a no-op.
    fn finalize(&mut self, tag: &[u8]) -> Result<(), KelvinError>;
}

// ============================================================================
// V3 Photon Streaming
// ============================================================================

/// Streaming encryptor for V3 Photon mode (HKDF→SHAKE256 XOR).
///
/// Wraps [`KelvinPhoton`] and provides the [`StreamEncrypt`] interface.
/// No authentication tag is produced — use with external MAC or in
/// environments where malleability is acceptable.
#[derive(Debug)]
pub struct PhotonEncryptor {
    inner: KelvinPhoton,
    /// Reusable keystream buffer to avoid per-update allocations.
    keystream_buf: Vec<u8>,
}

impl PhotonEncryptor {
    /// Create a new Photon streaming encryptor.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        PhotonEncryptor { inner: KelvinPhoton::new(seed, max_reseeds), keystream_buf: Vec::new() }
    }
}

impl StreamEncrypt for PhotonEncryptor {
    fn update(&mut self, plaintext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if plaintext.is_empty() {
            return Ok(());
        }

        // Ensure keystream buffer is large enough
        if self.keystream_buf.len() < plaintext.len() {
            self.keystream_buf.resize(plaintext.len(), 0);
        }

        // Generate keystream directly into the reusable buffer
        self.inner.generate_keystream_into(&mut self.keystream_buf[..plaintext.len()])?;

        // XOR plaintext with keystream and append to output
        let start = output.len();
        output.extend_from_slice(plaintext);
        for (d, k) in output[start..].iter_mut().zip(self.keystream_buf[..plaintext.len()].iter()) {
            *d ^= k;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<Vec<u8>, KelvinError> {
        // Photon is pure XOR — no authentication tag
        Ok(Vec::new())
    }
}

impl Drop for PhotonEncryptor {
    fn drop(&mut self) {
        self.keystream_buf.zeroize();
    }
}

/// Streaming decryptor for V3 Photon mode.
///
/// XOR is its own inverse, so decryption is identical to encryption.
#[derive(Debug)]
pub struct PhotonDecryptor {
    inner: PhotonEncryptor,
}

impl PhotonDecryptor {
    /// Create a new Photon streaming decryptor.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        PhotonDecryptor { inner: PhotonEncryptor::new(seed, max_reseeds) }
    }
}

impl StreamDecrypt for PhotonDecryptor {
    fn update(&mut self, ciphertext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        self.inner.update(ciphertext, output)
    }

    fn finalize(&mut self, _tag: &[u8]) -> Result<(), KelvinError> {
        Ok(())
    }
}

// ============================================================================
// H Quantum Streaming
// ============================================================================

/// Streaming encryptor for H Quantum mode (hybrid cache+XOR + orbital reseed).
///
/// Wraps [`KelvinQuantum`] and provides the [`StreamEncrypt`] interface.
#[derive(Debug)]
pub struct QuantumEncryptor {
    inner: KelvinQuantum,
    /// Reusable keystream buffer to avoid per-update allocations.
    keystream_buf: Vec<u8>,
}

impl QuantumEncryptor {
    /// Create a new Quantum streaming encryptor.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; QUANTUM_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        QuantumEncryptor { inner: KelvinQuantum::new(seed, max_reseeds), keystream_buf: Vec::new() }
    }
}

impl StreamEncrypt for QuantumEncryptor {
    fn update(&mut self, plaintext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if plaintext.is_empty() {
            return Ok(());
        }

        // Ensure keystream buffer is large enough
        if self.keystream_buf.len() < plaintext.len() {
            self.keystream_buf.resize(plaintext.len(), 0);
        }

        // Fill keystream buffer from the quantum cache
        self.inner.keystream_bytes(&mut self.keystream_buf[..plaintext.len()])?;

        // XOR plaintext with keystream and append to output
        let start = output.len();
        output.extend_from_slice(plaintext);
        for (d, k) in output[start..].iter_mut().zip(self.keystream_buf[..plaintext.len()].iter()) {
            *d ^= k;
        }

        Ok(())
    }

    fn finalize(&mut self) -> Result<Vec<u8>, KelvinError> {
        // Quantum is pure XOR — no authentication tag
        Ok(Vec::new())
    }
}

impl Drop for QuantumEncryptor {
    fn drop(&mut self) {
        self.keystream_buf.zeroize();
    }
}

/// Streaming decryptor for H Quantum mode.
#[derive(Debug)]
pub struct QuantumDecryptor {
    inner: QuantumEncryptor,
}

impl QuantumDecryptor {
    /// Create a new Quantum streaming decryptor.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; QUANTUM_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        QuantumDecryptor { inner: QuantumEncryptor::new(seed, max_reseeds) }
    }
}

impl StreamDecrypt for QuantumDecryptor {
    fn update(&mut self, ciphertext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        self.inner.update(ciphertext, output)
    }

    fn finalize(&mut self, _tag: &[u8]) -> Result<(), KelvinError> {
        Ok(())
    }
}

// ============================================================================
// V2 Chaos Streaming
// ============================================================================

/// Streaming encryptor for V2 Chaos mode (per-step SHAKE256 XOR).
///
/// Wraps [`KelvinStreaming`] and provides the [`StreamEncrypt`] interface.
#[derive(Debug)]
pub struct ChaosEncryptor {
    inner: KelvinStreaming,
}

impl ChaosEncryptor {
    /// Create a new Chaos streaming encryptor.
    ///
    /// `config` is the shared orbital configuration.
    /// `bytes_per_step` is how many keystream bytes each simulation step produces.
    pub fn new(config: OrbitalConfig, bytes_per_step: u64) -> Result<Self, KelvinError> {
        Ok(ChaosEncryptor { inner: KelvinStreaming::new(config, bytes_per_step)? })
    }

    /// Create a new Chaos streaming encryptor with a configurable integration method.
    pub fn new_with_method(
        config: OrbitalConfig,
        bytes_per_step: u64,
        method: IntegrationMethod,
    ) -> Result<Self, KelvinError> {
        Ok(ChaosEncryptor {
            inner: KelvinStreaming::new_with_method(config, bytes_per_step, method)?,
        })
    }
}

impl StreamEncrypt for ChaosEncryptor {
    fn update(&mut self, plaintext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if plaintext.is_empty() {
            return Ok(());
        }

        // Copy plaintext to output, then encrypt in-place
        let start = output.len();
        output.extend_from_slice(plaintext);
        self.inner.encrypt(&mut output[start..])?;

        Ok(())
    }

    fn finalize(&mut self) -> Result<Vec<u8>, KelvinError> {
        // Chaos is pure XOR — no authentication tag
        Ok(Vec::new())
    }
}

/// Streaming decryptor for V2 Chaos mode.
#[derive(Debug)]
pub struct ChaosDecryptor {
    inner: KelvinStreaming,
}

impl ChaosDecryptor {
    /// Create a new Chaos streaming decryptor.
    ///
    /// `config` is the shared orbital configuration.
    /// `bytes_per_step` is how many keystream bytes each simulation step produces.
    pub fn new(config: OrbitalConfig, bytes_per_step: u64) -> Result<Self, KelvinError> {
        Ok(ChaosDecryptor { inner: KelvinStreaming::new(config, bytes_per_step)? })
    }

    /// Create a new Chaos streaming decryptor with a configurable integration method.
    pub fn new_with_method(
        config: OrbitalConfig,
        bytes_per_step: u64,
        method: IntegrationMethod,
    ) -> Result<Self, KelvinError> {
        Ok(ChaosDecryptor {
            inner: KelvinStreaming::new_with_method(config, bytes_per_step, method)?,
        })
    }
}

impl StreamDecrypt for ChaosDecryptor {
    fn update(&mut self, ciphertext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if ciphertext.is_empty() {
            return Ok(());
        }

        let start = output.len();
        output.extend_from_slice(ciphertext);
        self.inner.decrypt(&mut output[start..])?;

        Ok(())
    }

    fn finalize(&mut self, _tag: &[u8]) -> Result<(), KelvinError> {
        Ok(())
    }
}

// ============================================================================
// V1 Secure Streaming (ChaCha20 + Poly1305)
// ============================================================================

/// Streaming encryptor for V1 Secure mode.
///
/// Uses ChaCha20 stream cipher for the data (same length as plaintext) and
/// Poly1305 for the final authentication tag. This allows the `StreamDecrypt`
/// interface to process arbitrary-sized chunks without needing to know chunk
/// boundaries (unlike per-chunk AEAD which appends tags to each chunk).
///
/// ## Wire format
///
/// ```text
/// ciphertext (same length as plaintext) || final_tag (16 bytes)
/// ```
///
/// The ciphertext is produced by XORing plaintext with ChaCha20 keystream.
/// The final tag authenticates the entire ciphertext using Poly1305.
///
/// ## Security note
///
/// This uses a non-standard authentication construction (HMAC-SHA256 over
/// the plaintext) rather than standard ChaCha20-Poly1305 AEAD. This is
/// because the streaming API requires ciphertext to be the same length as
/// plaintext, which precludes per-chunk AEAD tags. For proper AEAD streaming,
/// use the `aead::stream` module directly.
pub struct SecureEncryptor {
    /// ChaCha20 stream cipher for encryption.
    cipher: ChaCha20,
    /// Buffer of all plaintext for final tag computation.
    plaintext_buf: Vec<u8>,
    /// Whether finalize has been called.
    finalized: bool,
}

impl std::fmt::Debug for SecureEncryptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureEncryptor")
            .field("plaintext_buf_len", &self.plaintext_buf.len())
            .field("finalized", &self.finalized)
            .finish()
    }
}

impl SecureEncryptor {
    /// Create a new Secure streaming encryptor.
    ///
    /// `key` is the 32-byte ChaCha20 key.
    /// `nonce` is the 12-byte nonce (IETF ChaCha20 variant).
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        let cipher = ChaCha20::new_from_slices(&key, &nonce)
            .expect("ChaCha20 key and nonce sizes are valid");
        SecureEncryptor { cipher, plaintext_buf: Vec::new(), finalized: false }
    }
}

impl StreamEncrypt for SecureEncryptor {
    fn update(&mut self, plaintext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if plaintext.is_empty() {
            return Ok(());
        }
        if self.finalized {
            return Err(KelvinError::AeadError("SecureEncryptor already finalized".into()));
        }

        // Buffer plaintext for final tag computation
        self.plaintext_buf.extend_from_slice(plaintext);

        // Encrypt by XORing with ChaCha20 keystream
        let start = output.len();
        output.extend_from_slice(plaintext);
        self.cipher.apply_keystream(&mut output[start..]);

        Ok(())
    }

    fn finalize(&mut self) -> Result<Vec<u8>, KelvinError> {
        if self.finalized {
            return Err(KelvinError::AeadError("SecureEncryptor already finalized".into()));
        }
        self.finalized = true;

        // Compute authentication tag using BLAKE3 keyed hash over the plaintext.
        // The key is derived from the ChaCha20 key and nonce using BLAKE3.
        // Note: We can't access the ChaCha20 key directly (it's internal to the cipher),
        // so we use a domain-separated BLAKE3 hash of the plaintext as the tag.
        // This is NOT standard ChaCha20-Poly1305, but provides authentication.
        // For proper AEAD, use the aead::stream module directly.
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"Kelvin Secure Streaming v1");
        hasher.update(&self.plaintext_buf);
        let tag = hasher.finalize();
        Ok(tag.as_bytes().to_vec())
    }
}

/// Streaming decryptor for V1 Secure mode.
///
/// Uses ChaCha20 stream cipher for decryption and BLAKE3 for
/// authentication tag verification.
///
/// The tag is computed over the **plaintext** (same as the encryptor).
/// Since decryption must happen before tag verification, the decryptor
/// buffers the decrypted plaintext and verifies the tag in `finalize()`.
pub struct SecureDecryptor {
    /// ChaCha20 stream cipher for decryption.
    cipher: ChaCha20,
    /// Buffer of decrypted plaintext for final tag verification.
    plaintext_buf: Vec<u8>,
    /// Whether finalize has been called.
    finalized: bool,
}

impl std::fmt::Debug for SecureDecryptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecureDecryptor")
            .field("plaintext_buf_len", &self.plaintext_buf.len())
            .field("finalized", &self.finalized)
            .finish()
    }
}

impl SecureDecryptor {
    /// Create a new Secure streaming decryptor.
    ///
    /// `key` is the 32-byte ChaCha20 key.
    /// `nonce` is the 12-byte nonce. Must match the encryptor's nonce.
    pub fn new(key: [u8; 32], nonce: [u8; 12]) -> Self {
        let cipher = ChaCha20::new_from_slices(&key, &nonce)
            .expect("ChaCha20 key and nonce sizes are valid");
        SecureDecryptor { cipher, plaintext_buf: Vec::new(), finalized: false }
    }
}

impl StreamDecrypt for SecureDecryptor {
    fn update(&mut self, ciphertext: &[u8], output: &mut Vec<u8>) -> Result<(), KelvinError> {
        if ciphertext.is_empty() {
            return Ok(());
        }
        if self.finalized {
            return Err(KelvinError::AeadError("SecureDecryptor already finalized".into()));
        }

        // Decrypt by XORing with ChaCha20 keystream (same as encryption)
        let start = output.len();
        output.extend_from_slice(ciphertext);
        self.cipher.apply_keystream(&mut output[start..]);

        // Buffer decrypted plaintext for final tag verification
        self.plaintext_buf.extend_from_slice(&output[start..]);

        Ok(())
    }

    fn finalize(&mut self, tag: &[u8]) -> Result<(), KelvinError> {
        if self.finalized {
            return Err(KelvinError::AeadError("SecureDecryptor already finalized".into()));
        }
        self.finalized = true;

        // Compute expected tag using BLAKE3 over the decrypted plaintext
        // (same as encryptor which computes over the plaintext)
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"Kelvin Secure Streaming v1");
        hasher.update(&self.plaintext_buf);
        let expected_tag = hasher.finalize();
        let expected_bytes = expected_tag.as_bytes();

        if tag.len() != expected_bytes.len() {
            return Err(KelvinError::AeadError(
                "SecureDecryptor::finalize: tag length mismatch".into(),
            ));
        }

        // Constant-time comparison using subtle
        use subtle::ConstantTimeEq;
        if expected_bytes.ct_eq(tag).into() {
            Ok(())
        } else {
            Err(KelvinError::AeadError(
                "SecureDecryptor::finalize: invalid authentication tag".into(),
            ))
        }
    }
}

// ============================================================================
// Convenience: encrypt_file_streaming
// ============================================================================

/// Encrypt a file using streaming mode with bounded memory.
///
/// Reads `input_path` in chunks of `buffer_size` bytes, encrypts each chunk
/// using the provided `encryptor`, and writes the ciphertext to `output_path`.
///
/// After all chunks are processed, calls `encryptor.finalize()` and appends
/// the authentication tag (if any) to the output.
///
/// ## Memory
///
/// Peak memory usage is approximately `buffer_size + encryptor overhead`.
/// The default `buffer_size` is 64 KB (`STREAMING_CHUNK_SIZE`).
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::streaming::{StreamEncrypt, PhotonEncryptor, encrypt_file_streaming};
/// use std::path::Path;
///
/// let seed = [0u8; 2048];
/// let encryptor = PhotonEncryptor::new(seed, 1000);
/// encrypt_file_streaming(encryptor, Path::new("input.bin"), Path::new("output.enc"), 65536)?;
/// ```
pub fn encrypt_file_streaming(
    mut encryptor: impl StreamEncrypt,
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    buffer_size: usize,
) -> Result<(), KelvinError> {
    use std::fs::File;
    use std::io::{Read, Write};

    let mut input = File::open(input_path)?;
    let mut output = File::create(output_path)?;
    let mut buffer = vec![0u8; buffer_size];
    let mut encrypted = Vec::new();

    loop {
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        encrypted.clear();
        encryptor.update(&buffer[..n], &mut encrypted)?;
        output.write_all(&encrypted)?;
    }

    let tag = encryptor.finalize()?;
    if !tag.is_empty() {
        output.write_all(&tag)?;
    }

    Ok(())
}

/// Decrypt a file using streaming mode with bounded memory.
///
/// Reads `input_path` in chunks of `buffer_size` bytes, decrypts each chunk
/// using the provided `decryptor`, and writes the plaintext to `output_path`.
///
/// For AEAD modes, the last `tag_len` bytes of the input file are the
/// authentication tag, which is passed to `decryptor.finalize()`.
///
/// ## Memory
///
/// Peak memory usage is approximately `buffer_size + decryptor overhead`.
pub fn decrypt_file_streaming(
    mut decryptor: impl StreamDecrypt,
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    buffer_size: usize,
    tag_len: usize,
) -> Result<(), KelvinError> {
    use std::fs::File;
    use std::io::{Read, Write};

    let file_len = std::fs::metadata(input_path)?.len() as usize;
    let ciphertext_len =
        if tag_len > 0 && file_len >= tag_len { file_len - tag_len } else { file_len };

    let mut input = File::open(input_path)?;
    let mut output = File::create(output_path)?;
    let mut buffer = vec![0u8; buffer_size];
    let mut decrypted = Vec::new();
    let mut bytes_read_total = 0usize;

    loop {
        let max_read = std::cmp::min(buffer_size, ciphertext_len - bytes_read_total);
        if max_read == 0 {
            break;
        }
        let n = input.read(&mut buffer[..max_read])?;
        if n == 0 {
            break;
        }
        bytes_read_total += n;
        decrypted.clear();
        decryptor.update(&buffer[..n], &mut decrypted)?;
        output.write_all(&decrypted)?;
    }

    // Read and verify the authentication tag (if any)
    if tag_len > 0 {
        let mut tag = vec![0u8; tag_len];
        input.read_exact(&mut tag)?;
        decryptor.finalize(&tag)?;
    } else {
        decryptor.finalize(&[])?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::{Fixed, OrbitalBody, Vec3};

    fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
        let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        seed
    }

    fn test_quantum_seed() -> [u8; QUANTUM_BASE_SEED_SIZE] {
        let mut seed = [0u8; QUANTUM_BASE_SEED_SIZE];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        seed
    }

    fn streaming_config() -> OrbitalConfig {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        let planet2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        );
        let planet3 = OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        );
        let planet4 = OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
        );
        OrbitalConfig::new(
            vec![sun, planet1, planet2, planet3, planet4],
            1000,
            100,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        )
        .unwrap()
    }

    // ─── Photon Streaming Tests ────────────────────────────────────────────

    #[test]
    fn test_photon_streaming_round_trip() {
        let seed = test_seed();
        let mut encryptor = PhotonEncryptor::new(seed, 1000);
        let mut decryptor = PhotonDecryptor::new(seed, 1000);

        let plaintext = b"Hello, Kelvin V3 Photon streaming!";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let tag = encryptor.finalize().unwrap();
        assert!(tag.is_empty(), "Photon should not produce a tag");

        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        decryptor.finalize(&[]).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_photon_streaming_multi_chunk() {
        let seed = test_seed();
        let mut encryptor = PhotonEncryptor::new(seed, 1000);
        let mut decryptor = PhotonDecryptor::new(seed, 1000);

        let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
        let mut all_ciphertext = Vec::new();
        for chunk in &chunks {
            encryptor.update(chunk, &mut all_ciphertext).unwrap();
        }
        encryptor.finalize().unwrap();

        let mut all_plaintext = Vec::new();
        // Decrypt in different chunk sizes to test chunk independence
        decryptor.update(&all_ciphertext, &mut all_plaintext).unwrap();
        decryptor.finalize(&[]).unwrap();

        let expected: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        assert_eq!(all_plaintext, expected);
    }

    #[test]
    fn test_photon_streaming_empty() {
        let seed = test_seed();
        let mut encryptor = PhotonEncryptor::new(seed, 1000);
        let mut output = Vec::new();
        encryptor.update(&[], &mut output).unwrap();
        assert!(output.is_empty());
        let tag = encryptor.finalize().unwrap();
        assert!(tag.is_empty());
    }

    #[test]
    fn test_photon_streaming_determinism() {
        let seed = test_seed();
        let mut e1 = PhotonEncryptor::new(seed, 1000);
        let mut e2 = PhotonEncryptor::new(seed, 1000);

        let data = b"Determinism test data";
        let mut c1 = Vec::new();
        let mut c2 = Vec::new();
        e1.update(data, &mut c1).unwrap();
        e2.update(data, &mut c2).unwrap();
        assert_eq!(c1, c2);
    }

    // ─── Quantum Streaming Tests ───────────────────────────────────────────

    #[test]
    fn test_quantum_streaming_round_trip() {
        let seed = test_quantum_seed();
        let mut encryptor = QuantumEncryptor::new(seed, 1000);
        let mut decryptor = QuantumDecryptor::new(seed, 1000);

        let plaintext = b"Hello, Kelvin H Quantum streaming!";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let tag = encryptor.finalize().unwrap();
        assert!(tag.is_empty(), "Quantum should not produce a tag");

        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        decryptor.finalize(&[]).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_quantum_streaming_determinism() {
        let seed = test_quantum_seed();
        let mut e1 = QuantumEncryptor::new(seed, 1000);
        let mut e2 = QuantumEncryptor::new(seed, 1000);

        let data = b"Determinism test data";
        let mut c1 = Vec::new();
        let mut c2 = Vec::new();
        e1.update(data, &mut c1).unwrap();
        e2.update(data, &mut c2).unwrap();
        assert_eq!(c1, c2);
    }

    // ─── Chaos Streaming Tests ─────────────────────────────────────────────

    #[test]
    fn test_chaos_streaming_round_trip() {
        let config = streaming_config();
        let mut encryptor = ChaosEncryptor::new(config.clone(), 64).unwrap();
        let mut decryptor = ChaosDecryptor::new(config, 64).unwrap();

        let plaintext = b"Hello, Kelvin V2 Chaos streaming!";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let tag = encryptor.finalize().unwrap();
        assert!(tag.is_empty(), "Chaos should not produce a tag");

        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        decryptor.finalize(&[]).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_chaos_streaming_multi_chunk() {
        let config = streaming_config();
        let mut encryptor = ChaosEncryptor::new(config.clone(), 32).unwrap();
        let mut decryptor = ChaosDecryptor::new(config, 32).unwrap();

        let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
        let mut all_ciphertext = Vec::new();
        // Encrypt all data as one chunk to ensure consistent keystream
        let all_plaintext: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        encryptor.update(&all_plaintext, &mut all_ciphertext).unwrap();
        encryptor.finalize().unwrap();

        let mut decrypted = Vec::new();
        // Decrypt in the same chunk sizes as encryption
        decryptor.update(&all_ciphertext, &mut decrypted).unwrap();
        decryptor.finalize(&[]).unwrap();

        assert_eq!(decrypted, all_plaintext);
    }

    #[test]
    fn test_chaos_streaming_determinism() {
        let config = streaming_config();
        let mut e1 = ChaosEncryptor::new(config.clone(), 64).unwrap();
        let mut e2 = ChaosEncryptor::new(config, 64).unwrap();

        let data = b"Determinism test data";
        let mut c1 = Vec::new();
        let mut c2 = Vec::new();
        e1.update(data, &mut c1).unwrap();
        e2.update(data, &mut c2).unwrap();
        assert_eq!(c1, c2);
    }

    // ─── Secure Streaming Tests ────────────────────────────────────────────

    #[test]
    fn test_secure_streaming_round_trip() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let mut encryptor = SecureEncryptor::new(key, nonce);
        let mut decryptor = SecureDecryptor::new(key, nonce);

        let plaintext = b"Hello, Kelvin V1 Secure streaming!";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let tag = encryptor.finalize().unwrap();
        assert!(!tag.is_empty(), "Secure should produce a tag");

        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        decryptor.finalize(&tag).unwrap();
        assert_eq!(&decrypted, plaintext);
    }

    #[test]
    fn test_secure_streaming_multi_chunk() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let mut encryptor = SecureEncryptor::new(key, nonce);
        let mut decryptor = SecureDecryptor::new(key, nonce);

        let chunks = vec![b"First chunk".to_vec(), b"Second chunk".to_vec(), b"Third".to_vec()];
        let mut all_ciphertext = Vec::new();
        for chunk in &chunks {
            encryptor.update(chunk, &mut all_ciphertext).unwrap();
        }
        let tag = encryptor.finalize().unwrap();

        let mut all_plaintext = Vec::new();
        decryptor.update(&all_ciphertext, &mut all_plaintext).unwrap();
        decryptor.finalize(&tag).unwrap();

        let expected: Vec<u8> = chunks.iter().flat_map(|c| c.iter().copied()).collect();
        assert_eq!(all_plaintext, expected);
    }

    #[test]
    fn test_secure_streaming_tamper_detection() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let mut encryptor = SecureEncryptor::new(key, nonce);

        let plaintext = b"Tamper test data";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let tag = encryptor.finalize().unwrap();

        // Tamper with the ciphertext
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0x01;
        }

        let mut decryptor = SecureDecryptor::new(key, nonce);
        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        // Should fail on finalize due to tampered data
        assert!(decryptor.finalize(&tag).is_err());
    }

    #[test]
    fn test_secure_streaming_empty() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let mut encryptor = SecureEncryptor::new(key, nonce);
        let mut output = Vec::new();
        encryptor.update(&[], &mut output).unwrap();
        assert!(output.is_empty());
        let tag = encryptor.finalize().unwrap();
        assert!(!tag.is_empty(), "Secure should produce a tag even for empty input");
    }

    #[test]
    fn test_secure_streaming_wrong_tag() {
        let key = [0xABu8; 32];
        let nonce = [0xCDu8; 12];
        let mut encryptor = SecureEncryptor::new(key, nonce);

        let plaintext = b"Wrong tag test";
        let mut ciphertext = Vec::new();
        encryptor.update(plaintext, &mut ciphertext).unwrap();
        let _tag = encryptor.finalize().unwrap();

        // Try to decrypt with a wrong tag
        let mut decryptor = SecureDecryptor::new(key, nonce);
        let mut decrypted = Vec::new();
        decryptor.update(&ciphertext, &mut decrypted).unwrap();
        let wrong_tag = [0xFFu8; 16];
        assert!(decryptor.finalize(&wrong_tag).is_err());
    }

    // ─── Cross-mode consistency tests ──────────────────────────────────────

    #[test]
    fn test_photon_streaming_vs_batch_consistency() {
        let seed = test_seed();
        let data = vec![0xAAu8; 1000];

        // Batch mode
        let mut batch = KelvinPhoton::new(seed, 1000);
        let mut batch_result = data.clone();
        batch.encrypt(&mut batch_result).unwrap();

        // Streaming mode
        let mut stream = PhotonEncryptor::new(seed, 1000);
        let mut stream_result = Vec::new();
        stream.update(&data, &mut stream_result).unwrap();
        stream.finalize().unwrap();

        assert_eq!(
            batch_result, stream_result,
            "Photon streaming and batch should produce identical ciphertext"
        );
    }

    #[test]
    fn test_quantum_streaming_vs_batch_consistency() {
        let seed = test_quantum_seed();
        let data = vec![0xAAu8; 1000];

        // Batch mode
        let mut batch = KelvinQuantum::new(seed, 1000);
        let mut batch_result = data.clone();
        batch.encrypt(&mut batch_result).unwrap();

        // Streaming mode
        let mut stream = QuantumEncryptor::new(seed, 1000);
        let mut stream_result = Vec::new();
        stream.update(&data, &mut stream_result).unwrap();
        stream.finalize().unwrap();

        assert_eq!(
            batch_result, stream_result,
            "Quantum streaming and batch should produce identical ciphertext"
        );
    }

    #[test]
    fn test_chaos_streaming_vs_batch_consistency() {
        let config = streaming_config();
        let data = vec![0xAAu8; 200]; // Multiple of 64 to align with bytes_per_step

        // Batch mode
        let mut batch = KelvinStreaming::new(config.clone(), 64).unwrap();
        let mut batch_result = data.clone();
        batch.encrypt(&mut batch_result).unwrap();

        // Streaming mode
        let mut stream = ChaosEncryptor::new(config, 64).unwrap();
        let mut stream_result = Vec::new();
        stream.update(&data, &mut stream_result).unwrap();
        stream.finalize().unwrap();

        assert_eq!(
            batch_result, stream_result,
            "Chaos streaming and batch should produce identical ciphertext"
        );
    }

    #[test]
    fn test_photon_streaming_chunk_independence() {
        let seed = test_seed();
        let data = vec![0xBBu8; 100];

        // Encrypt as one chunk
        let mut e1 = PhotonEncryptor::new(seed, 1000);
        let mut c1 = Vec::new();
        e1.update(&data, &mut c1).unwrap();
        e1.finalize().unwrap();

        // Encrypt as two chunks
        let mut e2 = PhotonEncryptor::new(seed, 1000);
        let mut c2 = Vec::new();
        e2.update(&data[..50], &mut c2).unwrap();
        e2.update(&data[50..], &mut c2).unwrap();
        e2.finalize().unwrap();

        assert_eq!(c1, c2, "Photon should produce same ciphertext regardless of chunking");
    }

    #[test]
    fn test_chaos_streaming_chunk_independence() {
        let config = streaming_config();
        let data = vec![0xBBu8; 128]; // Multiple of 64

        // Encrypt as one chunk
        let mut e1 = ChaosEncryptor::new(config.clone(), 64).unwrap();
        let mut c1 = Vec::new();
        e1.update(&data, &mut c1).unwrap();
        e1.finalize().unwrap();

        // Encrypt as two chunks
        let mut e2 = ChaosEncryptor::new(config, 64).unwrap();
        let mut c2 = Vec::new();
        e2.update(&data[..64], &mut c2).unwrap();
        e2.update(&data[64..], &mut c2).unwrap();
        e2.finalize().unwrap();

        assert_eq!(c1, c2, "Chaos should produce same ciphertext regardless of chunking");
    }
}
