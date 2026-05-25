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
//! KMAC128(key, ciphertext, customization_string) → 32-byte tag
//! ```
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
use kelvin_core::{Fixed, OrbitalBody};
use kelvin_kdf::{extract_shake256_into, OrbitalConfig};

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

/// Compute a KMAC128 tag over the ciphertext.
///
/// Uses the `sha3-kmac` crate (NIST SP 800-185 compliant KMAC128).
/// The tag is 32 bytes (256 bits), providing 128-bit security against forgery.
fn compute_tag(key: &[u8; MAC_KEY_SIZE], ciphertext: &[u8], custom: &[u8]) -> [u8; TAG_LEN] {
    use sha3_kmac::generic_array::typenum::U32;
    use sha3_kmac::Mac;

    let mut mac = Kmac128::new(key, custom).expect("KMAC128 key size is valid");
    mac.update(ciphertext);
    let tag: Mac<U32> = mac.finalize();
    let mut out = [0u8; TAG_LEN];
    out.copy_from_slice(&tag);
    out
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
/// let mut auth = KelvinPhotonAuthenticated::new(seed, 1000)?;
/// let mut data = b"Secret message".to_vec();
/// auth.encrypt(&mut data)?;
/// auth.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
#[derive(Debug)]
pub struct KelvinPhotonAuthenticated {
    /// Inner Photon instance for keystream generation.
    inner: KelvinPhoton,
    /// KMAC128 key derived from the seed.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
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
        }
    }

    /// Encrypt data in-place with authentication.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let plaintext_len = data.len() - AUTH_OVERHEAD;
        let (plaintext, auth_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Write version byte
        auth_out[0] = AUTH_FORMAT_VERSION;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinPhotonAuthenticated-v1");
        auth_out[1..].copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// The buffer must contain ciphertext + version byte + 32-byte
    /// authentication tag. Returns an error if the tag doesn't match
    /// (tampered data) or the version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - AUTH_OVERHEAD;
        let (ciphertext, auth_in) = data.split_at_mut(ciphertext_len);

        // Check version byte
        if auth_in[0] != AUTH_FORMAT_VERSION {
            return Err(KelvinError::AuthenticationFailed(format!(
                "unsupported auth format version: expected 0x{:02x}, got 0x{:02x}",
                AUTH_FORMAT_VERSION, auth_in[0]
            )));
        }

        // Verify the tag before decrypting (constant-time comparison)
        let expected_tag = compute_tag(&self.mac_key, ciphertext, b"KelvinPhotonAuthenticated-v1");
        if !bool::from(expected_tag.ct_eq(&auth_in[1..])) {
            return Err(KelvinError::AuthenticationFailed("KMAC128 tag mismatch".into()));
        }

        // Decrypt the ciphertext in-place
        self.inner.decrypt(ciphertext)?;

        Ok(())
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
/// let mut auth = KelvinQuantumAuthenticated::new(seed, 1000)?;
/// let mut data = b"Secret message".to_vec();
/// auth.encrypt(&mut data)?;
/// auth.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
#[derive(Debug)]
pub struct KelvinQuantumAuthenticated {
    /// Inner Quantum instance for keystream generation.
    inner: KelvinQuantum,
    /// KMAC128 key derived from the seed.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
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
        }
    }

    /// Encrypt data in-place with authentication.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let plaintext_len = data.len() - AUTH_OVERHEAD;
        let (plaintext, auth_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Write version byte
        auth_out[0] = AUTH_FORMAT_VERSION;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinQuantumAuthenticated-v1");
        auth_out[1..].copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// Returns an error if the tag doesn't match (tampered data) or the
    /// version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - AUTH_OVERHEAD;
        let (ciphertext, auth_in) = data.split_at_mut(ciphertext_len);

        // Check version byte
        if auth_in[0] != AUTH_FORMAT_VERSION {
            return Err(KelvinError::AuthenticationFailed(format!(
                "unsupported auth format version: expected 0x{:02x}, got 0x{:02x}",
                AUTH_FORMAT_VERSION, auth_in[0]
            )));
        }

        // Verify the tag before decrypting (constant-time comparison)
        let expected_tag = compute_tag(&self.mac_key, ciphertext, b"KelvinQuantumAuthenticated-v1");
        if !bool::from(expected_tag.ct_eq(&auth_in[1..])) {
            return Err(KelvinError::AuthenticationFailed("KMAC128 tag mismatch".into()));
        }

        // Decrypt the ciphertext in-place
        self.inner.decrypt(ciphertext)?;

        Ok(())
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
/// let mut auth = KelvinStreamingAuthenticated::new(config, 1024 * 1024)?;
/// let mut data = b"Secret message".to_vec();
/// auth.encrypt(&mut data)?;
/// auth.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
#[derive(Debug)]
pub struct KelvinStreamingAuthenticated {
    /// Inner streaming instance for keystream generation.
    inner: crate::KelvinStreaming,
    /// KMAC128 key derived from the initial orbital state.
    mac_key: [u8; MAC_KEY_SIZE],
    /// Buffer for computing authentication tags.
    tag_buf: [u8; TAG_LEN],
}

/// Derive a MAC key from the initial orbital bodies for V2 streaming mode.
///
/// Uses SHAKE256 to extract entropy from the bodies, then HKDF-SHA512 to
/// derive the 32-byte KMAC128 key.
fn derive_mac_key_from_bodies(
    bodies: &[OrbitalBody],
    step: u64,
    g: Fixed,
    softening: Fixed,
) -> [u8; MAC_KEY_SIZE] {
    let mut extract_buf = [0u8; EXTRACT_BUF_SIZE];
    extract_shake256_into(
        bodies,
        step,
        g,
        softening,
        DOMSEP_STREAMING_MAC_KEY_V1,
        &mut extract_buf,
    );

    let hk = Hkdf::<Sha3_512>::new(None, &extract_buf);
    let mut mac_key = [0u8; MAC_KEY_SIZE];
    hk.expand(b"kelvin-streaming-mac-key-v1", &mut mac_key)
        .expect("HKDF expand with 32-byte output should never fail");
    mac_key
}

impl KelvinStreamingAuthenticated {
    /// Create a new authenticated streaming instance.
    ///
    /// `config` is the shared orbital configuration.
    /// `bytes_per_step` is how many keystream bytes each simulation step produces.
    pub fn new(config: OrbitalConfig, bytes_per_step: u64) -> Result<Self, KelvinError> {
        // We need the initial bodies to derive the MAC key.
        // Clone them from the config before creating the streaming instance.
        let bodies = config.bodies.clone();

        let inner = crate::KelvinStreaming::new(config, bytes_per_step)?;

        // Derive MAC key from initial bodies (step 0)
        let mac_key = derive_mac_key_from_bodies(
            &bodies,
            0,
            kelvin_core::DEFAULT_G,
            kelvin_core::SOFTENING_FACTOR,
        );

        Ok(KelvinStreamingAuthenticated { inner, mac_key, tag_buf: [0u8; TAG_LEN] })
    }

    /// Encrypt data in-place with authentication.
    ///
    /// The buffer must have `AUTH_OVERHEAD` (33) extra bytes after the
    /// plaintext for the version byte + KMAC128 authentication tag.
    ///
    /// Wire format: `ciphertext (N bytes) || version (1 byte) || KMAC128 tag (32 bytes)`
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let plaintext_len = data.len() - AUTH_OVERHEAD;
        let (plaintext, auth_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Write version byte
        auth_out[0] = AUTH_FORMAT_VERSION;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinStreamingAuthenticated-v1");
        auth_out[1..].copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// Returns an error if the tag doesn't match (tampered data) or the
    /// version is unknown.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < AUTH_OVERHEAD {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for auth overhead, got {}",
                AUTH_OVERHEAD,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - AUTH_OVERHEAD;
        let (ciphertext, auth_in) = data.split_at_mut(ciphertext_len);

        // Check version byte
        if auth_in[0] != AUTH_FORMAT_VERSION {
            return Err(KelvinError::AuthenticationFailed(format!(
                "unsupported auth format version: expected 0x{:02x}, got 0x{:02x}",
                AUTH_FORMAT_VERSION, auth_in[0]
            )));
        }

        // Verify the tag before decrypting (constant-time comparison)
        let expected_tag =
            compute_tag(&self.mac_key, ciphertext, b"KelvinStreamingAuthenticated-v1");
        if !bool::from(expected_tag.ct_eq(&auth_in[1..])) {
            return Err(KelvinError::AuthenticationFailed("KMAC128 tag mismatch".into()));
        }

        // Decrypt the ciphertext in-place
        self.inner.decrypt(ciphertext)?;

        Ok(())
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
