//! Authenticated wrappers for V3 Photon and H Quantum stream modes.
//!
//! Both V3 (`KelvinPhoton`) and H (`KelvinQuantum`) are pure XOR stream
//! ciphers with no built-in authentication. This module provides wrappers
//! that append a **BLAKE3-keyed MAC tag** (32 bytes) to the ciphertext,
//! and verify it in constant time before decryption.
//!
//! ## Wire Format
//!
//! ```text
//! ciphertext (N bytes) || BLAKE3-keyed MAC tag (32 bytes)
//! ```
//!
//! ## MAC Key Derivation
//!
//! A dedicated 32-byte MAC key is derived from the 2048-byte seed using
//! HKDF-SHA512 with domain separator `b"kelvin-mac-key-v1"`. This key is
//! stored separately from the inner crypto object so it is not consumed
//! during reseeding.
//!
//! ## Security
//!
//! - **Constant-time verification**: Uses `subtle::ConstantTimeEq` to
//!   compare MAC tags, defeating timing side-channel attacks.
//! - **Separate MAC key**: Domain-separated from the encryption keystream.
//! - **BLAKE3-keyed MAC**: BLAKE3 in keyed mode is a native PRF-MAC,
//!   providing 128-bit security against quantum adversaries.
//!
//! ## Example
//!
//! ```rust,ignore
//! use kelvin::KelvinPhotonAuthenticated;
//!
//! let seed = [0u8; 2048]; // From orbital simulation
//! let mut auth = KelvinPhotonAuthenticated::new(seed, 1000);
//!
//! let mut data = b"Secret message".to_vec();
//! auth.encrypt(&mut data)?; // data now has 32 extra bytes (MAC tag)
//! auth.decrypt(&mut data)?; // tag verified, then stripped
//! assert_eq!(&data, b"Secret message");
//! ```

use blake3::Hash;
use hkdf::Hkdf;
use sha3::Sha3_512;
use subtle::ConstantTimeEq;
use zeroize::Zeroize;

use crate::error::KelvinError;
use crate::photon::KelvinPhoton;
use crate::quantum::KelvinQuantum;

/// Size of the BLAKE3-keyed MAC tag in bytes.
const TAG_LEN: usize = 32;

/// Derive a 32-byte MAC key from the 2048-byte seed using HKDF-SHA512.
///
/// Domain separator: `b"kelvin-mac-key-v1"`.
fn derive_mac_key(seed: &[u8; 2048]) -> [u8; 32] {
    let hk = Hkdf::<Sha3_512>::new(None, seed);
    let mut mac_key = [0u8; 32];
    hk.expand(b"kelvin-mac-key-v1", &mut mac_key)
        .expect("HKDF expand with 32-byte output should never fail");
    mac_key
}

/// Compute a BLAKE3-keyed MAC tag over `data` using `key`.
fn compute_tag(key: &[u8; 32], data: &[u8]) -> Hash {
    blake3::keyed_hash(key, data)
}

/// Verify `tag` against a freshly computed MAC of `data` using `key`.
///
/// Uses `subtle::ConstantTimeEq` to prevent timing side-channels.
fn verify_tag(key: &[u8; 32], data: &[u8], tag: &[u8; 32]) -> Result<(), KelvinError> {
    let expected = compute_tag(key, data);
    if expected.as_bytes().ct_eq(tag).into() {
        Ok(())
    } else {
        Err(KelvinError::AuthenticationFailed(
            "BLAKE3-keyed MAC tag mismatch".to_string(),
        ))
    }
}

// ============================================================================
// KelvinPhotonAuthenticated
// ============================================================================

/// Authenticated wrapper around [`KelvinPhoton`].
///
/// Appends a 32-byte BLAKE3-keyed MAC tag to the ciphertext on encryption,
/// and verifies it in constant time before decryption.
///
/// ## Wire Format
///
/// ```text
/// ciphertext (N bytes) || BLAKE3-keyed MAC tag (32 bytes)
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::KelvinPhotonAuthenticated;
///
/// let seed = [0u8; 2048];
/// let mut auth = KelvinPhotonAuthenticated::new(seed, 1000);
///
/// let mut data = b"Hello, authenticated Photon!".to_vec();
/// auth.encrypt(&mut data).unwrap();
/// // data.len() is now original_len + 32
/// auth.decrypt(&mut data).unwrap();
/// assert_eq!(&data, b"Hello, authenticated Photon!");
/// ```
#[derive(Debug)]
pub struct KelvinPhotonAuthenticated {
    /// Inner V3 Photon engine.
    inner: KelvinPhoton,
    /// Dedicated 32-byte MAC key (derived from seed, not consumed by reseeding).
    mac_key: [u8; 32],
}

impl KelvinPhotonAuthenticated {
    /// Create a new authenticated Photon instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    pub fn new(seed: [u8; 2048], max_reseeds: u64) -> Self {
        let mac_key = derive_mac_key(&seed);
        KelvinPhotonAuthenticated {
            inner: KelvinPhoton::new(seed, max_reseeds),
            mac_key,
        }
    }

    /// Encrypt data in-place and append a 32-byte MAC tag.
    ///
    /// The input `data` Vec is grown by 32 bytes to accommodate the tag.
    pub fn encrypt(&mut self, data: &mut Vec<u8>) -> Result<(), KelvinError> {
        // Encrypt the plaintext in-place (XOR with keystream)
        self.inner.encrypt(data.as_mut_slice())?;

        // Compute MAC over the ciphertext and append
        let tag = compute_tag(&self.mac_key, data);
        data.extend_from_slice(tag.as_bytes());

        Ok(())
    }

    /// Verify the MAC tag and decrypt data in-place.
    ///
    /// The last 32 bytes of `data` are treated as the MAC tag. If verification
    /// fails, `AuthenticationFailed` is returned and `data` is **not** modified.
    /// On success, the tag is stripped and the plaintext remains in `data`.
    pub fn decrypt(&mut self, data: &mut Vec<u8>) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::AuthenticationFailed(
                "ciphertext too short to contain MAC tag".to_string(),
            ));
        }

        let split_point = data.len() - TAG_LEN;
        let (ciphertext, tag_bytes) = data.split_at(split_point);

        // Constant-time verify the tag before decrypting
        let mut tag_arr = [0u8; TAG_LEN];
        tag_arr.copy_from_slice(tag_bytes);
        verify_tag(&self.mac_key, ciphertext, &tag_arr)?;

        // Tag verified — now decrypt (XOR is its own inverse)
        // Truncate the tag first, then decrypt the ciphertext in-place
        data.truncate(split_point);
        self.inner.decrypt(data.as_mut_slice())?;

        Ok(())
    }

    /// Get the total bytes processed (plaintext bytes, excluding tags).
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
    }
}

// ============================================================================
// KelvinQuantumAuthenticated
// ============================================================================

/// Authenticated wrapper around [`KelvinQuantum`].
///
/// Appends a 32-byte BLAKE3-keyed MAC tag to the ciphertext on encryption,
/// and verifies it in constant time before decryption.
///
/// ## Wire Format
///
/// ```text
/// ciphertext (N bytes) || BLAKE3-keyed MAC tag (32 bytes)
/// ```
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::KelvinQuantumAuthenticated;
///
/// let seed = [0u8; 2048];
/// let mut auth = KelvinQuantumAuthenticated::new(seed, 1000);
///
/// let mut data = b"Hello, authenticated Quantum!".to_vec();
/// auth.encrypt(&mut data).unwrap();
/// // data.len() is now original_len + 32
/// auth.decrypt(&mut data).unwrap();
/// assert_eq!(&data, b"Hello, authenticated Quantum!");
/// ```
#[derive(Debug)]
pub struct KelvinQuantumAuthenticated {
    /// Inner H Quantum engine.
    inner: KelvinQuantum,
    /// Dedicated 32-byte MAC key (derived from seed, not consumed by reseeding).
    mac_key: [u8; 32],
}

impl KelvinQuantumAuthenticated {
    /// Create a new authenticated Quantum instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    ///
    /// # Panics
    ///
    /// Panics if the initial keystream cache refill fails (e.g., `max_reseeds` is 0).
    pub fn new(seed: [u8; 2048], max_reseeds: u64) -> Self {
        let mac_key = derive_mac_key(&seed);
        KelvinQuantumAuthenticated {
            inner: KelvinQuantum::new(seed, max_reseeds),
            mac_key,
        }
    }

    /// Create a new authenticated Quantum instance with custom configuration.
    ///
    /// Returns an error if the initial keystream cache refill fails.
    pub fn with_config(
        seed: [u8; 2048],
        max_reseeds: u64,
        cache_size: usize,
        orbital_steps_per_reseed: u64,
        reseed_interval_bytes: u64,
    ) -> Result<Self, KelvinError> {
        let mac_key = derive_mac_key(&seed);
        let inner = KelvinQuantum::with_config(
            seed,
            max_reseeds,
            cache_size,
            orbital_steps_per_reseed,
            reseed_interval_bytes,
        )?;
        Ok(KelvinQuantumAuthenticated { inner, mac_key })
    }

    /// Encrypt data in-place and append a 32-byte MAC tag.
    ///
    /// The input `data` Vec is grown by 32 bytes to accommodate the tag.
    pub fn encrypt(&mut self, data: &mut Vec<u8>) -> Result<(), KelvinError> {
        // Encrypt the plaintext in-place (XOR with keystream)
        self.inner.encrypt(data.as_mut_slice())?;

        // Compute MAC over the ciphertext and append
        let tag = compute_tag(&self.mac_key, data);
        data.extend_from_slice(tag.as_bytes());

        Ok(())
    }

    /// Verify the MAC tag and decrypt data in-place.
    ///
    /// The last 32 bytes of `data` are treated as the MAC tag. If verification
    /// fails, `AuthenticationFailed` is returned and `data` is **not** modified.
    /// On success, the tag is stripped and the plaintext remains in `data`.
    pub fn decrypt(&mut self, data: &mut Vec<u8>) -> Result<(), KelvinError> {
        if data.len() < TAG_LEN {
            return Err(KelvinError::AuthenticationFailed(
                "ciphertext too short to contain MAC tag".to_string(),
            ));
        }

        let split_point = data.len() - TAG_LEN;
        let (ciphertext, tag_bytes) = data.split_at(split_point);

        // Constant-time verify the tag before decrypting
        let mut tag_arr = [0u8; TAG_LEN];
        tag_arr.copy_from_slice(tag_bytes);
        verify_tag(&self.mac_key, ciphertext, &tag_arr)?;

        // Tag verified — now decrypt (XOR is its own inverse)
        // Truncate the tag first, then decrypt the ciphertext in-place
        data.truncate(split_point);
        self.inner.decrypt(data.as_mut_slice())?;

        Ok(())
    }

    /// Get the total bytes generated (plaintext bytes, excluding tags).
    pub fn bytes_generated(&self) -> u64 {
        self.inner.bytes_generated()
    }

    /// Get the current reseed count.
    pub fn reseed_count(&self) -> u64 {
        self.inner.reseed_count()
    }

    /// Get the remaining reseeds before exhaustion.
    pub fn remaining_reseeds(&self) -> u64 {
        self.inner.remaining_reseeds()
    }

    /// Get the current orbital step.
    pub fn orbital_step(&self) -> u64 {
        self.inner.orbital_step()
    }
}

impl Drop for KelvinQuantumAuthenticated {
    fn drop(&mut self) {
        self.mac_key.zeroize();
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; 2048] {
        let mut seed = [0u8; 2048];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        seed
    }

    // ─── Photon Authenticated Tests ────────────────────────────────────────

    #[test]
    fn test_photon_auth_round_trip() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = b"Hello, authenticated Photon!".to_vec();
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        // Data should be 32 bytes longer (MAC tag appended)
        assert_eq!(data.len(), original.len() + TAG_LEN);
        assert_ne!(&data[..original.len()], &original[..]);

        // Decrypt with new instance
        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(data, original, "round-trip should restore original");
    }

    #[test]
    fn test_photon_auth_tampered_ciphertext() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = b"Tamper test data".to_vec();
        auth.encrypt(&mut data).unwrap();

        // Flip a bit in the ciphertext (not the tag)
        data[5] ^= 0x01;

        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let result = auth2.decrypt(&mut data);
        assert!(
            result.is_err(),
            "tampered ciphertext should fail authentication"
        );
        match result {
            Err(KelvinError::AuthenticationFailed(_)) => {} // expected
            _ => panic!("expected AuthenticationFailed error"),
        }
    }

    #[test]
    fn test_photon_auth_tampered_tag() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = b"Tamper tag test".to_vec();
        auth.encrypt(&mut data).unwrap();

        // Flip a bit in the tag (last 32 bytes)
        let tag_start = data.len() - TAG_LEN;
        data[tag_start + 10] ^= 0x01;

        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let result = auth2.decrypt(&mut data);
        assert!(
            result.is_err(),
            "tampered tag should fail authentication"
        );
        match result {
            Err(KelvinError::AuthenticationFailed(_)) => {} // expected
            _ => panic!("expected AuthenticationFailed error"),
        }
    }

    #[test]
    fn test_photon_auth_determinism() {
        let mut a1 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut a2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);

        let mut data1 = b"Determinism test".to_vec();
        let mut data2 = data1.clone();

        a1.encrypt(&mut data1).unwrap();
        a2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "two instances should produce identical ciphertext+tag");
    }

    #[test]
    fn test_photon_auth_empty_data() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data: Vec<u8> = vec![];

        auth.encrypt(&mut data).unwrap();
        // Empty plaintext → 32-byte tag on wire
        assert_eq!(data.len(), TAG_LEN);

        let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert!(data.is_empty(), "decrypted empty data should be empty");
    }

    #[test]
    fn test_photon_auth_too_short_for_tag() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        let mut data = vec![0u8; 16]; // Less than TAG_LEN
        let result = auth.decrypt(&mut data);
        assert!(result.is_err(), "data shorter than tag should fail");
    }

    #[test]
    fn test_photon_auth_bytes_processed() {
        let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
        assert_eq!(auth.bytes_processed(), 0);

        let mut data = b"Hello".to_vec();
        auth.encrypt(&mut data).unwrap();
        // bytes_processed should count plaintext, not tag
        assert_eq!(auth.bytes_processed(), 5);
    }

    // ─── Quantum Authenticated Tests ───────────────────────────────────────

    #[test]
    fn test_quantum_auth_round_trip() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut data = b"Hello, authenticated Quantum!".to_vec();
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        assert_eq!(data.len(), original.len() + TAG_LEN);
        assert_ne!(&data[..original.len()], &original[..]);

        let mut auth2 = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(data, original, "round-trip should restore original");
    }

    #[test]
    fn test_quantum_auth_tampered_ciphertext() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut data = b"Quantum tamper test".to_vec();
        auth.encrypt(&mut data).unwrap();

        data[3] ^= 0x01;

        let mut auth2 = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let result = auth2.decrypt(&mut data);
        assert!(result.is_err(), "tampered ciphertext should fail authentication");
        match result {
            Err(KelvinError::AuthenticationFailed(_)) => {}
            _ => panic!("expected AuthenticationFailed error"),
        }
    }

    #[test]
    fn test_quantum_auth_tampered_tag() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut data = b"Quantum tag tamper".to_vec();
        auth.encrypt(&mut data).unwrap();

        let tag_start = data.len() - TAG_LEN;
        data[tag_start + 5] ^= 0x01;

        let mut auth2 = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let result = auth2.decrypt(&mut data);
        assert!(result.is_err(), "tampered tag should fail authentication");
        match result {
            Err(KelvinError::AuthenticationFailed(_)) => {}
            _ => panic!("expected AuthenticationFailed error"),
        }
    }

    #[test]
    fn test_quantum_auth_determinism() {
        let mut a1 = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut a2 = KelvinQuantumAuthenticated::new(test_seed(), 1000);

        let mut data1 = b"Quantum determinism".to_vec();
        let mut data2 = data1.clone();

        a1.encrypt(&mut data1).unwrap();
        a2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "two instances should produce identical ciphertext+tag");
    }

    #[test]
    fn test_quantum_auth_empty_data() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut data: Vec<u8> = vec![];

        auth.encrypt(&mut data).unwrap();
        assert_eq!(data.len(), TAG_LEN);

        let mut auth2 = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        auth2.decrypt(&mut data).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_quantum_auth_too_short_for_tag() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        let mut data = vec![0u8; 8];
        let result = auth.decrypt(&mut data);
        assert!(result.is_err(), "data shorter than tag should fail");
    }

    #[test]
    fn test_quantum_auth_with_config() {
        let mut auth = KelvinQuantumAuthenticated::with_config(test_seed(), 1000, 64, 10, 128)
            .expect("with_config should succeed");
        let mut data = b"Config test data".to_vec();
        let original = data.clone();

        auth.encrypt(&mut data).unwrap();
        assert_eq!(data.len(), original.len() + TAG_LEN);

        let mut auth2 = KelvinQuantumAuthenticated::with_config(test_seed(), 1000, 64, 10, 128)
            .expect("with_config should succeed");
        auth2.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_quantum_auth_reseed_preserved() {
        // Verify that authenticated wrapper preserves reseed behavior
        let mut auth = KelvinQuantumAuthenticated::with_config(test_seed(), 1000, 64, 10, 128)
            .expect("with_config should succeed");
        let mut data = vec![0u8; 1000];
        auth.encrypt(&mut data).unwrap();
        assert!(
            auth.reseed_count() > 0,
            "should have triggered reseeds with small interval"
        );
    }

    #[test]
    fn test_quantum_auth_bytes_generated() {
        let mut auth = KelvinQuantumAuthenticated::new(test_seed(), 1000);
        assert_eq!(auth.bytes_generated(), 0);

        let mut data = b"Hello".to_vec();
        auth.encrypt(&mut data).unwrap();
        assert_eq!(auth.bytes_generated(), 5);
    }

    // ─── Cross-mode tests ──────────────────────────────────────────────────

    #[test]
    fn test_photon_and_quantum_different_tags() {
        // Same seed, different modes → different keystreams → different tags
        let seed = test_seed();

        let mut pa = KelvinPhotonAuthenticated::new(seed, 1000);
        let mut qa = KelvinQuantumAuthenticated::new(seed, 1000);

        let mut data_p = b"Cross-mode test".to_vec();
        let mut data_q = b"Cross-mode test".to_vec();

        pa.encrypt(&mut data_p).unwrap();
        qa.encrypt(&mut data_q).unwrap();

        assert_ne!(data_p, data_q, "Photon and Quantum should produce different outputs");
    }
}
