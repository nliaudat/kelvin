//! Authenticated wrappers for V3 Photon, H Quantum, and V2 Streaming modes.
//!
//! These wrappers add NIST SP 800-185 KMAC128 authentication on top of the
//! base XOR-based encryption, providing both confidentiality and integrity.
//!
//! ## Architecture
//!
//! Each authenticated wrapper derives a KMAC128 key from the same seed used
//! for keystream generation, using HKDF-SHA512 with a domain separator:
//!
//! ```text
//! the 2048-byte seed using HKDF-SHA512 with domain separator
//!   ↓
//! 32-byte KMAC128 key
//!   ↓
//! KMAC128(key, version || chunk index || final flag || ciphertext,
//!         customization_string) → 32-byte tag
//! ```
//!
//! ## Chunked Streams
//!
//! Every tag is bound to the position of its chunk in the stream, so chunks
//! cannot be reordered, repeated or skipped. The sender seals the last chunk
//! (which may be empty) with `encrypt_final`, and the receiver opens it with
//! `decrypt_final`. A receiver that reaches the end of its input without a
//! successful `decrypt_final` must treat the stream as truncated.
//!
//! One instance handles one stream in one direction: use separate instances to
//! encrypt and to decrypt.
//!
//! ## Security
//!
//! - **Authenticated encryption**: KMAC128 provides 128-bit security against
//!   forgery (NIST SP 800-185).
//! - **Domain separation**: MAC key derivation uses a distinct domain separator
//!   from keystream generation, preventing related-key attacks.
//! - **Quantum-resistant**: KMAC128 is based on SHAKE256, providing 256-bit
//!   classical / 128-bit quantum security.
//!
//! ## References
//!
//! - NIST SP 800-185 (2016). "SHA-3 Derived Functions: cSHAKE, KMAC, TupleHash,
//!   and ParallelHash."
//! - Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key
//!   Derivation Function (HKDF)." RFC 5869.

use hkdf::Hkdf;
use sha3::Sha3_512;
use sha3_kmac::Kmac128;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use crate::error::KelvinError;
use crate::parameters::{
    AUTH_FORMAT_VERSION, AUTH_OVERHEAD, AUTH_TAG_LEN, DOMSEP_MAC_KEY_V1,
    DOMSEP_STREAMING_MAC_KEY_V1, EXTRACT_BUF_SIZE, MAC_KEY_SIZE, PHOTON_BASE_SEED_SIZE,
    QUANTUM_BASE_SEED_SIZE,
};
use crate::photon::KelvinPhoton;
use crate::quantum::KelvinQuantum;
use kelvin_core::{Fixed, IntegrationMethod, OrbitalBody};
use kelvin_kdf::{extract_shake256_into_with_nonce, OrbitalConfig};

/// Size of the KMAC128 tag in bytes.
const TAG_LEN: usize = AUTH_TAG_LEN;

/// Derive a MAC key from the 2048-byte seed using HKDF-SHA512.
///
/// Domain separator: `DOMSEP_MAC_KEY_V1`.
fn derive_mac_key(seed: &[u8; PHOTON_BASE_SEED_SIZE]) -> [u8; MAC_KEY_SIZE] {
    let hk = Hkdf::<Sha3_512>::new(None, seed);
    let mut mac_key = [0u8; MAC_KEY_SIZE];
    hk.expand(DOMSEP_MAC_KEY_V1, &mut mac_key)
        .expect("HKDF expand with 32-byte output should never fail");
    mac_key
}

/// Position of an authenticated stream: next chunk index and whether the
/// final chunk has been processed.
#[derive(Debug, Default)]
struct ChunkState {
    next: u64,
    finished: bool,
}

/// Compute a KMAC128 tag over one chunk.
///
/// Uses the `sha3-kmac` crate (NIST SP 800-185 compliant KMAC128).
/// The tag is 32 bytes (256 bits), providing 128-bit security against forgery.
///
/// The MAC input is `version (1) || chunk index (8, LE) || final flag (1) ||
/// ciphertext`. The prefix has a fixed length, so the encoding is unambiguous.
fn compute_tag(
    key: &[u8; MAC_KEY_SIZE],
    index: u64,
    last: bool,
    ciphertext: &[u8],
    custom: &[u8],
) -> [u8; TAG_LEN] {
    use sha3_kmac::generic_array::typenum::U32;
    use sha3_kmac::Mac;

    let mut mac = Kmac128::new(key, custom).expect("KMAC128 key size is valid");
    mac.update(&[AUTH_FORMAT_VERSION]);
    mac.update(&index.to_le_bytes());
    mac.update(&[u8::from(last)]);
    mac.update(ciphertext);
    let tag: Mac<U32> = mac.finalize();
    let mut out = [0u8; TAG_LEN];
    out.copy_from_slice(&tag);
    out
}

/// Check that the stream is still open and the buffer can hold the version
/// byte and tag. Returns the length of the chunk without that overhead.
fn check_chunk(state: &ChunkState, len: usize) -> Result<usize, KelvinError> {
    if state.finished {
        return Err(KelvinError::AuthenticationFailed(
            "stream already ended with a final chunk".into(),
        ));
    }
    len.checked_sub(AUTH_OVERHEAD).ok_or_else(|| {
        KelvinError::InvalidConfig(format!(
            "buffer too short: need at least {} bytes for auth overhead, got {}",
            AUTH_OVERHEAD, len
        ))
    })
}

/// Encrypt one chunk in place and append the version byte and tag.
fn seal_chunk(
    state: &mut ChunkState,
    key: &[u8; MAC_KEY_SIZE],
    custom: &[u8],
    data: &mut [u8],
    last: bool,
    encrypt: impl FnOnce(&mut [u8]) -> Result<(), KelvinError>,
) -> Result<(), KelvinError> {
    let plaintext_len = check_chunk(state, data.len())?;
    let (plaintext, auth_out) = data.split_at_mut(plaintext_len);

    // Encrypt the plaintext in-place
    encrypt(plaintext)?;

    // Write the version byte, then the tag over the ciphertext
    auth_out[0] = AUTH_FORMAT_VERSION;
    let tag = compute_tag(key, state.next, last, plaintext, custom);
    auth_out[1..].copy_from_slice(&tag);

    state.next += 1;
    state.finished = last;
    Ok(())
}

/// Verify one chunk and decrypt it in place. Nothing is decrypted and the
/// stream does not advance unless the tag matches.
fn open_chunk(
    state: &mut ChunkState,
    key: &[u8; MAC_KEY_SIZE],
    custom: &[u8],
    data: &mut [u8],
    last: bool,
    decrypt: impl FnOnce(&mut [u8]) -> Result<(), KelvinError>,
) -> Result<(), KelvinError> {
    let ciphertext_len = check_chunk(state, data.len())?;
    let (ciphertext, auth_in) = data.split_at_mut(ciphertext_len);

    // Check version byte
    if auth_in[0] != AUTH_FORMAT_VERSION {
        return Err(KelvinError::AuthenticationFailed(format!(
            "unsupported auth format version: expected 0x{:02x}, got 0x{:02x}",
            AUTH_FORMAT_VERSION, auth_in[0]
        )));
    }

    // Verify the tag before decrypting (constant-time comparison)
    let expected_tag = compute_tag(key, state.next, last, ciphertext, custom);
    if !bool::from(expected_tag.ct_eq(&auth_in[1..])) {
        return Err(KelvinError::AuthenticationFailed("KMAC128 tag mismatch".into()));
    }

    // Decrypt the ciphertext in-place
    decrypt(ciphertext)?;

    state.next += 1;
    state.finished = last;
    Ok(())
}

// ============================================================================
// V3 Photon Authenticated
// ============================================================================

/// Authenticated V3 Kelvin-Photon: KMAC128 + XOR keystream.
///
/// Provides both confidentiality (XOR with SHAKE256 keystream) and
/// integrity (KMAC128 authentication tag).
///
/// ## Usage
///
/// ```rust,ignore
/// use kelvin::KelvinPhotonAuthenticated;
///
/// let seed = [0u8; PHOTON_BASE_SEED_SIZE];
/// let mut sender = KelvinPhotonAuthenticated::new(seed, 1000);
/// let mut receiver = KelvinPhotonAuthenticated::new(seed, 1000);
/// let mut data = b"Secret message".to_vec();
/// data.resize(data.len() + AUTH_OVERHEAD, 0);
/// sender.encrypt_final(&mut data)?;
/// receiver.decrypt_final(&mut data)?; // a second instance
/// assert_eq!(&data[..data.len() - AUTH_OVERHEAD], b"Secret message");
/// ```
pub struct KelvinPhotonAuthenticated {
    /// Inner Photon instance for keystream generation.
    inner: KelvinPhoton,
    /// KMAC128 key derived from the seed.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
    /// Chunk counter and final-chunk state bound into each tag.
    chunks: ChunkState,
}

/// Secret material is omitted from the output.
impl core::fmt::Debug for KelvinPhotonAuthenticated {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KelvinPhotonAuthenticated")
            .field("inner", &self.inner)
            .field("chunks", &self.chunks)
            .finish_non_exhaustive()
    }
}

impl KelvinPhotonAuthenticated {
    /// Create a new authenticated Photon instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; PHOTON_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        let mac_key = derive_mac_key(&seed);
        KelvinPhotonAuthenticated {
            inner: KelvinPhoton::new(seed, max_reseeds),
            mac_key,
            tag_buf: [0u8; TAG_LEN],
            chunks: ChunkState::default(),
        }
    }

    /// Encrypt a chunk in-place with authentication. Use
    /// [`encrypt_final`](Self::encrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, false)
    }

    /// Encrypt the last chunk of a stream (the plaintext may be empty).
    /// This instance cannot process further chunks afterwards.
    pub fn encrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, true)
    }

    /// Verify and decrypt a chunk in-place. Use
    /// [`decrypt_final`](Self::decrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must contain ciphertext + version byte + 32-byte
    /// authentication tag. Returns an error if the tag does not match
    /// (tampered, reordered or repeated chunk) or the version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, false)
    }

    /// Verify and decrypt the last chunk of a stream. It succeeds only for a
    /// chunk sealed with `encrypt_final`, which is how truncation is detected.
    pub fn decrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, true)
    }

    fn seal(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        seal_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinPhotonAuthenticated-v2",
            data,
            last,
            |buf| inner.encrypt(buf),
        )
    }

    fn open(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        open_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinPhotonAuthenticated-v2",
            data,
            last,
            |buf| inner.decrypt(buf),
        )
    }

    /// Get the total bytes processed (plaintext only, excluding tags).
    pub fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    /// Get the current reseed count.
    pub fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    /// Get the remaining reseeds before exhaustion.
    pub fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

impl Drop for KelvinPhotonAuthenticated {
    fn drop(&mut self) {
        self.mac_key.zeroize();
        self.tag_buf.zeroize();
    }
}

// ============================================================================
// H Quantum Authenticated
// ============================================================================

/// Authenticated H Kelvin-Quantum: KMAC128 + XOR keystream.
///
/// Provides both confidentiality (XOR with SHAKE256 keystream) and
/// integrity (KMAC128 authentication tag).
///
/// ## Usage
///
/// ```rust,ignore
/// use kelvin::KelvinQuantumAuthenticated;
///
/// let seed = [0u8; QUANTUM_BASE_SEED_SIZE];
/// let mut sender = KelvinQuantumAuthenticated::new(seed, 1000);
/// let mut receiver = KelvinQuantumAuthenticated::new(seed, 1000);
/// let mut data = b"Secret message".to_vec();
/// data.resize(data.len() + AUTH_OVERHEAD, 0);
/// sender.encrypt_final(&mut data)?;
/// receiver.decrypt_final(&mut data)?; // a second instance
/// assert_eq!(&data[..data.len() - AUTH_OVERHEAD], b"Secret message");
/// ```
pub struct KelvinQuantumAuthenticated {
    /// Inner Quantum instance for keystream generation.
    inner: KelvinQuantum,
    /// KMAC128 key derived from the seed.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
    /// Chunk counter and final-chunk state bound into each tag.
    chunks: ChunkState,
}

/// Secret material is omitted from the output.
impl core::fmt::Debug for KelvinQuantumAuthenticated {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KelvinQuantumAuthenticated")
            .field("inner", &self.inner)
            .field("chunks", &self.chunks)
            .finish_non_exhaustive()
    }
}

impl KelvinQuantumAuthenticated {
    /// Create a new authenticated Quantum instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; QUANTUM_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        let mac_key = derive_mac_key(&seed);
        KelvinQuantumAuthenticated {
            inner: KelvinQuantum::new(seed, max_reseeds),
            mac_key,
            tag_buf: [0u8; TAG_LEN],
            chunks: ChunkState::default(),
        }
    }

    /// Encrypt a chunk in-place with authentication. Use
    /// [`encrypt_final`](Self::encrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, false)
    }

    /// Encrypt the last chunk of a stream (the plaintext may be empty).
    /// This instance cannot process further chunks afterwards.
    pub fn encrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, true)
    }

    /// Verify and decrypt a chunk in-place. Use
    /// [`decrypt_final`](Self::decrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must contain ciphertext + version byte + 32-byte
    /// authentication tag. Returns an error if the tag does not match
    /// (tampered, reordered or repeated chunk) or the version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, false)
    }

    /// Verify and decrypt the last chunk of a stream. It succeeds only for a
    /// chunk sealed with `encrypt_final`, which is how truncation is detected.
    pub fn decrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, true)
    }

    fn seal(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        seal_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinQuantumAuthenticated-v2",
            data,
            last,
            |buf| inner.encrypt(buf),
        )
    }

    fn open(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        open_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinQuantumAuthenticated-v2",
            data,
            last,
            |buf| inner.decrypt(buf),
        )
    }

    /// Get the total bytes processed (plaintext only, excluding tags).
    pub fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    /// Get the current reseed count.
    pub fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    /// Get the remaining reseeds before exhaustion.
    pub fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }
}

impl Drop for KelvinQuantumAuthenticated {
    fn drop(&mut self) {
        self.mac_key.zeroize();
        self.tag_buf.zeroize();
    }
}

// ============================================================================
// V2 Streaming Authenticated
// ============================================================================

/// Authenticated V2 Kelvin-Streaming: KMAC128 + XOR keystream.
///
/// Unlike V3 Photon and H Quantum (which derive the MAC key from a 2048-byte
/// seed), V2 Streaming derives the MAC key directly from the initial orbital
/// body state via SHAKE256 extraction.
///
/// ## Usage
///
/// ```rust,ignore
/// use kelvin::{KelvinStreamingAuthenticated, OrbitalConfig};
///
/// let config = OrbitalConfig::from_json(json_str)?;
/// let mut sender = KelvinStreamingAuthenticated::new(config.clone(), 1024 * 1024)?;
/// let mut receiver = KelvinStreamingAuthenticated::new(config, 1024 * 1024)?;
/// let mut data = b"Secret message".to_vec();
/// data.resize(data.len() + AUTH_OVERHEAD, 0);
/// sender.encrypt_final(&mut data)?;
/// receiver.decrypt_final(&mut data)?; // a second instance
/// assert_eq!(&data[..data.len() - AUTH_OVERHEAD], b"Secret message");
/// ```
pub struct KelvinStreamingAuthenticated {
    /// Inner streaming instance for keystream generation.
    inner: crate::KelvinStreaming,
    /// KMAC128 key derived from the initial orbital state.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
    /// Chunk counter and final-chunk state bound into each tag.
    chunks: ChunkState,
}

/// Secret material is omitted from the output.
impl core::fmt::Debug for KelvinStreamingAuthenticated {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KelvinStreamingAuthenticated")
            .field("inner", &self.inner)
            .field("chunks", &self.chunks)
            .finish_non_exhaustive()
    }
}

/// Derive a MAC key from the initial orbital bodies for V2 streaming mode.
///
/// Uses SHAKE256 to extract entropy from the bodies, then HKDF-SHA512 to
/// derive the 32-byte KMAC128 key.
///
/// The tag covers ciphertext, so a receiver whose settings produce a different
/// keystream must also get a different MAC key; otherwise it would accept the
/// chunk and return garbage. The integration method, time step and bytes per
/// step are therefore part of the HKDF info.
fn derive_mac_key_from_bodies(
    bodies: &[OrbitalBody],
    step: u64,
    g: Fixed,
    softening: Fixed,
    nonce: &[u8],
    method: IntegrationMethod,
    dt: Fixed,
    bytes_per_step: u64,
) -> [u8; MAC_KEY_SIZE] {
    let mut extract_buf = [0u8; EXTRACT_BUF_SIZE];
    extract_shake256_into_with_nonce(
        bodies,
        step,
        g,
        softening,
        DOMSEP_STREAMING_MAC_KEY_V1,
        nonce,
        &mut extract_buf,
    );

    let hk = Hkdf::<Sha3_512>::new(None, &extract_buf);
    let mut mac_key = [0u8; MAC_KEY_SIZE];
    let method_id: u8 = match method {
        IntegrationMethod::Verlet => 0,
        IntegrationMethod::Euler => 1,
    };
    let info: [&[u8]; 4] = [
        b"kelvin-streaming-mac-key-v2",
        &[method_id],
        &dt.to_raw().to_le_bytes(),
        &bytes_per_step.to_le_bytes(),
    ];
    hk.expand_multi_info(&info, &mut mac_key)
        .expect("HKDF expand with 32-byte output should never fail");
    mac_key
}

impl KelvinStreamingAuthenticated {
    /// Create a new authenticated streaming instance.
    ///
    /// `config` is the shared orbital configuration.
    /// `bytes_per_step` is how many keystream bytes each simulation step produces.
    pub fn new(config: OrbitalConfig, bytes_per_step: u64) -> Result<Self, KelvinError> {
        Self::new_with_nonce(config, bytes_per_step, &[])
    }

    /// Create an authenticated streaming instance bound to a session `nonce`.
    ///
    /// The nonce is bound into both the keystream and the KMAC128 key, so
    /// nonce-separated instances are fully independent. See
    /// [`crate::KelvinStreaming::new_with_nonce`].
    pub fn new_with_nonce(
        config: OrbitalConfig,
        bytes_per_step: u64,
        nonce: &[u8],
    ) -> Result<Self, KelvinError> {
        Self::new_with_method_and_nonce(config, bytes_per_step, IntegrationMethod::default(), nonce)
    }

    /// Like [`new_with_nonce`](Self::new_with_nonce) with a configurable
    /// integration method. Both parties must use the same method.
    pub fn new_with_method_and_nonce(
        config: OrbitalConfig,
        bytes_per_step: u64,
        method: IntegrationMethod,
        nonce: &[u8],
    ) -> Result<Self, KelvinError> {
        // We need the initial bodies to derive the MAC key.
        // Clone them from the config before creating the streaming instance.
        let bodies = config.bodies.clone();
        let (g, softening, dt) = (config.g, config.softening, config.dt);

        let inner = crate::KelvinStreaming::new_with_method_and_nonce(
            config,
            bytes_per_step,
            method,
            nonce,
        )?;

        // Derive MAC key from initial bodies (step 0), bound to the nonce
        let mac_key = derive_mac_key_from_bodies(
            &bodies,
            0,
            g,
            softening,
            nonce,
            method,
            dt,
            inner.bytes_per_step(),
        );

        Ok(KelvinStreamingAuthenticated {
            inner,
            mac_key,
            tag_buf: [0u8; TAG_LEN],
            chunks: ChunkState::default(),
        })
    }

    /// Encrypt a chunk in-place with authentication. Use
    /// [`encrypt_final`](Self::encrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, false)
    }

    /// Encrypt the last chunk of a stream (the plaintext may be empty).
    /// This instance cannot process further chunks afterwards.
    pub fn encrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.seal(data, true)
    }

    /// Verify and decrypt a chunk in-place. Use
    /// [`decrypt_final`](Self::decrypt_final) for the last chunk of a stream.
    ///
    /// The buffer must contain ciphertext + version byte + 32-byte
    /// authentication tag. Returns an error if the tag does not match
    /// (tampered, reordered or repeated chunk) or the version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, false)
    }

    /// Verify and decrypt the last chunk of a stream. It succeeds only for a
    /// chunk sealed with `encrypt_final`, which is how truncation is detected.
    pub fn decrypt_final(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.open(data, true)
    }

    fn seal(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        seal_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinStreamingAuthenticated-v2",
            data,
            last,
            |buf| inner.encrypt(buf),
        )
    }

    fn open(&mut self, data: &mut [u8], last: bool) -> Result<(), KelvinError> {
        let inner = &mut self.inner;
        open_chunk(
            &mut self.chunks,
            &self.mac_key,
            b"KelvinStreamingAuthenticated-v2",
            data,
            last,
            |buf| inner.decrypt(buf),
        )
    }

    /// Get the total bytes processed (plaintext only, excluding tags).
    pub fn bytes_processed(&self) -> u64 {
        self.inner.bytes_processed()
    }

    /// Get the current step counter.
    pub fn step(&self) -> u64 {
        self.inner.step()
    }
}

impl Drop for KelvinStreamingAuthenticated {
    fn drop(&mut self) {
        self.mac_key.zeroize();
        self.tag_buf.zeroize();
    }
}
