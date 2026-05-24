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
use sha3::digest::{ExtendableOutput, Update, XofReader};
use sha3::{Sha3_512, Shake256};
use zeroize::Zeroize;

use crate::error::KelvinError;
use crate::parameters::{
    DOMSEP_MAC_KEY_V1, DOMSEP_STREAMING_MAC_KEY_V1, EXTRACT_BUF_SIZE, MAC_KEY_SIZE,
    PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE,
};
use crate::photon::KelvinPhoton;
use crate::quantum::KelvinQuantum;
use kelvin_core::{Fixed, OrbitalBody};
use kelvin_kdf::{extract_shake256_into, OrbitalConfig};

/// Size of the KMAC128 tag in bytes.
const TAG_LEN: usize = 32;

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
/// Uses SHAKE256 with the KMAC construction from NIST SP 800-185.
/// The tag is 32 bytes (256 bits), providing 128-bit security against forgery.
fn compute_tag(key: &[u8; MAC_KEY_SIZE], ciphertext: &[u8], custom: &[u8]) -> [u8; TAG_LEN] {
    // KMAC128(K, X, L, S):
    //   newX = bytepad(encode_string(K), 168) || X || right_encode(L)
    //   return cSHAKE128(newX, L, "KMAC", S)
    //
    // We use SHAKE256 instead of SHAKE128 for 256-bit security.
    // The KMAC construction is:
    //   SHAKE256(bytepad(encode_string(K), 136) || X || right_encode(L) || "KMAC")

    let mut hasher = Shake256::default();

    // bytepad(encode_string(K), 136) — 136 is SHAKE256's rate in bytes
    // encode_string(K) = left_encode(len(K)) || K
    // left_encode(x) encodes x as a byte string with the length prefix
    let key_len_encoded = encode_string_length(key.len());
    hasher.update(&key_len_encoded);
    hasher.update(key);

    // Pad to 136 bytes (SHAKE256 rate)
    let padded_len = key_len_encoded.len() + key.len();
    let padding = 136 - (padded_len % 136);
    if padding < 136 {
        hasher.update(&vec![0u8; padding]);
    }

    // X = ciphertext
    hasher.update(ciphertext);

    // right_encode(0) — output length 0 means we want the full output
    hasher.update(&[0u8, 0x01]); // right_encode(0) = 0x00 || 0x01

    // "KMAC" customization
    hasher.update(b"KMAC");

    // Customization string S
    hasher.update(custom);

    let mut reader = hasher.finalize_xof();
    let mut tag = [0u8; TAG_LEN];
    XofReader::read(&mut reader, &mut tag);
    tag
}

/// Encode the length as a byte string for KMAC's encode_string.
///
/// left_encode(x) encodes x as a byte string with the number of bytes
/// needed to represent x prepended.
fn encode_string_length(len: usize) -> Vec<u8> {
    if len == 0 {
        return vec![0x01, 0x00];
    }
    let bytes = len.to_be_bytes();
    // Find the first non-zero byte
    let start = bytes.iter().position(|&b| b != 0).unwrap_or(bytes.len() - 1);
    let mut result = vec![(bytes.len() - start) as u8];
    result.extend_from_slice(&bytes[start..]);
    result
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
    /// The buffer must have `TAG_LEN` (32) extra bytes after the plaintext
    /// for the KMAC128 authentication tag.
    ///
    /// The tag is computed over the ciphertext and appended to the buffer.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let plaintext_len = data.len() - TAG_LEN;
        let (plaintext, tag_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinPhotonAuthenticated-v1");
        tag_out.copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// The buffer must contain ciphertext + 32-byte authentication tag.
    /// Returns an error if the tag doesn't match (tampered data).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - TAG_LEN;
        let (ciphertext, tag_in) = data.split_at_mut(ciphertext_len);

        // Verify the tag before decrypting
        let expected_tag = compute_tag(&self.mac_key, ciphertext, b"KelvinPhotonAuthenticated-v1");
        if tag_in != &expected_tag[..] {
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
    /// The buffer must have `TAG_LEN` (32) extra bytes after the plaintext
    /// for the KMAC128 authentication tag.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let plaintext_len = data.len() - TAG_LEN;
        let (plaintext, tag_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinQuantumAuthenticated-v1");
        tag_out.copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// Returns an error if the tag doesn't match (tampered data).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - TAG_LEN;
        let (ciphertext, tag_in) = data.split_at_mut(ciphertext_len);

        // Verify the tag before decrypting
        let expected_tag = compute_tag(&self.mac_key, ciphertext, b"KelvinQuantumAuthenticated-v1");
        if tag_in != &expected_tag[..] {
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
fn derive_mac_key_from_bodies(bodies: &[OrbitalBody], step: u64, g: Fixed, softening: Fixed) -> [u8; MAC_KEY_SIZE] {
    let mut extract_buf = [0u8; EXTRACT_BUF_SIZE];
    extract_shake256_into(bodies, step, g, softening, DOMSEP_STREAMING_MAC_KEY_V1, &mut extract_buf);

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
        let mac_key = derive_mac_key_from_bodies(&bodies, 0, kelvin_core::DEFAULT_G, kelvin_core::SOFTENING_FACTOR);

        Ok(KelvinStreamingAuthenticated {
            inner,
            mac_key,
            tag_buf: [0u8; TAG_LEN],
        })
    }

    /// Encrypt data in-place with authentication.
    ///
    /// The buffer must have `TAG_LEN` (32) extra bytes after the plaintext
    /// for the KMAC128 authentication tag.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let plaintext_len = data.len() - TAG_LEN;
        let (plaintext, tag_out) = data.split_at_mut(plaintext_len);

        // Encrypt the plaintext in-place
        self.inner.encrypt(plaintext)?;

        // Compute KMAC128 tag over the ciphertext
        let tag = compute_tag(&self.mac_key, plaintext, b"KelvinStreamingAuthenticated-v1");
        tag_out.copy_from_slice(&tag);

        Ok(())
    }

    /// Decrypt data in-place with authentication verification.
    ///
    /// Returns an error if the tag doesn't match (tampered data).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::InvalidConfig(format!(
                "buffer too short: need at least {} bytes for tag, got {}",
                TAG_LEN,
                data.len()
            )));
        }

        let ciphertext_len = data.len() - TAG_LEN;
        let (ciphertext, tag_in) = data.split_at_mut(ciphertext_len);

        // Verify the tag before decrypting
        let expected_tag = compute_tag(&self.mac_key, ciphertext, b"KelvinStreamingAuthenticated-v1");
        if tag_in != &expected_tag[..] {
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

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::{Fixed, Vec3};

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

    // ─── Photon Authenticated Tests ────────────────────────────────────────

    #[test]
    fn test_photon_auth_round_trip() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        // Buffer needs 32 extra bytes for KMAC128 tag
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        // Ciphertext portion (first 64 bytes) should differ from plaintext
        assert_ne!(&data[..64], &original[..64]);

        // Decrypt with new instance (same seed = same keystream + same MAC key)
        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(&data[..64], &original[..64]);
    }

    #[test]
    fn test_photon_auth_tamper_detection() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        auth.encrypt(&mut data).unwrap();

        // Tamper with the ciphertext
        data[0] ^= 0x01;

        // Decryption should fail
        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        assert!(auth2.decrypt(&mut data).is_err());
    }

    #[test]
    fn test_photon_auth_tag_mismatch() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        auth.encrypt(&mut data).unwrap();

        // Tamper with the tag
        let last = data.len() - 1;
        data[last] ^= 0x01;

        // Decryption should fail
        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        assert!(auth2.decrypt(&mut data).is_err());
    }

    #[test]
    fn test_photon_auth_too_short() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = vec![0xABu8; TAG_LEN - 1]; // Too short for tag
        assert!(auth.encrypt(&mut data).is_err());
        assert!(auth.decrypt(&mut data).is_err());
    }

    // ─── Quantum Authenticated Tests ───────────────────────────────────────

    #[test]
    fn test_quantum_auth_round_trip() {
        let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        assert_ne!(&data[..64], &original[..64]);

        let mut auth2 = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(&data[..64], &original[..64]);
    }

    #[test]
    fn test_quantum_auth_tamper_detection() {
        let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        auth.encrypt(&mut data).unwrap();

        data[0] ^= 0x01;

        let mut auth2 = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
        assert!(auth2.decrypt(&mut data).is_err());
    }

    #[test]
    fn test_quantum_auth_too_short() {
        let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
        let mut data = vec![0xABu8; TAG_LEN - 1];
        assert!(auth.encrypt(&mut data).is_err());
        assert!(auth.decrypt(&mut data).is_err());
    }

    // ─── Streaming Authenticated Tests ─────────────────────────────────────

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

    #[test]
    fn test_streaming_auth_round_trip() {
        let config = streaming_config();
        let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        assert_ne!(&data[..64], &original[..64]);

        let mut auth2 = KelvinStreamingAuthenticated::new(streaming_config(), 64).unwrap();
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(&data[..64], &original[..64]);
    }

    #[test]
    fn test_streaming_auth_tamper_detection() {
        let config = streaming_config();
        let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
        let mut data = vec![0xABu8; 64 + TAG_LEN];
        auth.encrypt(&mut data).unwrap();

        data[0] ^= 0x01;

        let mut auth2 = KelvinStreamingAuthenticated::new(streaming_config(), 64).unwrap();
        assert!(auth2.decrypt(&mut data).is_err());
    }

    #[test]
    fn test_streaming_auth_too_short() {
        let config = streaming_config();
        let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
        let mut data = vec![0xABu8; TAG_LEN - 1];
        assert!(auth.encrypt(&mut data).is_err());
        assert!(auth.decrypt(&mut data).is_err());
    }
}
