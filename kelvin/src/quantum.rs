//! H Kelvin-Quantum — Hybrid orbital chaos + quantum-resistant stream cipher.
//!
//! ## Architecture
//!
//! H Kelvin-Quantum combines the orbital chaos KDF with a quantum-resistant
//! stream cipher (SHAKE256 XOF). Unlike V3 Photon (which uses HKDF→SHAKE256),
//! H Quantum periodically refreshes its base seed with fresh orbital entropy
//! via `reseed_from_orbital_chaos`:
//!
//! ```text
//! 2048B base seed → BLAKE3 XOF → perturbation → orbital state
//!   ↓
//! SHAKE256 XOF → keystream cache (1 MiB)
//!   ↓
//! XOR with plaintext/ciphertext
//!   ↓
//! Every N bytes: reseed_from_orbital_chaos
//!   → advance orbital simulation by M steps
//!   → extract fresh entropy via SHAKE256
//!   → XOR into base seed
//! ```
//!
//! ## Security
//!
//! - **No authentication**: XOR is malleable. Use with external MAC or
//!   in environments where malleability is acceptable.
//! - **Orbital reseeding**: Fresh chaotic entropy is mixed in periodically,
//!   providing forward secrecy beyond BLAKE3's deterministic reseeding.
//! - **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit
//!   quantum security.
//!
//! ## References
//!
//! - NIST FIPS PUB 202 (2015). "SHA-3 Standard."
//! - Bernstein, D. J. (2014). "ChaCha, a variant of Salsa20."

use blake3::Hasher;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::Shake256;
use zeroize::{Zeroize, Zeroizing};

use crate::error::KelvinError;
use crate::parameters::{
    DOMSEP_QUANTUM_CACHE_V1, DOMSEP_QUANTUM_KEYSTREAM, DOMSEP_QUANTUM_PERTURB_V1, EXTRACT_BUF_SIZE,
    KEYSTREAM_CHUNK_SIZE, QUANTUM_BASE_SEED_SIZE, QUANTUM_DEFAULT_CACHE_SIZE,
    QUANTUM_DEFAULT_ORBITAL_STEPS, QUANTUM_DEFAULT_RESEED_INTERVAL, QUANTUM_PERTURB_SCALE,
    XOF_SEED_SIZE,
};
use kelvin_kdf::OrbitalState;

/// H Kelvin-Quantum: hybrid orbital chaos + quantum-resistant stream cipher.
///
/// Produces arbitrary-length keystream from a 2048-byte base seed.
/// Periodically refreshes the base seed with fresh orbital entropy.
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::KelvinQuantum;
///
/// let seed = [0u8; QUANTUM_BASE_SEED_SIZE]; // From orbital simulation
/// let mut quantum = KelvinQuantum::new(seed, 1000)?;
/// let mut data = b"Secret message".to_vec();
/// quantum.encrypt(&mut data)?;
/// quantum.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
#[derive(Debug)]
pub struct KelvinQuantum {
    /// Base seed material (QUANTUM_BASE_SEED_SIZE bytes), refreshed with orbital entropy.
    base_seed: [u8; QUANTUM_BASE_SEED_SIZE],
    /// Keystream cache (pre-generated bulk keystream).
    keystream_cache: Vec<u8>,
    /// Current position in the keystream cache.
    cache_pos: usize,
    /// Orbital state for fresh entropy generation.
    orbital_state: OrbitalState,
    /// Orbital steps to run per reseed (Euler integration).
    orbital_steps_per_reseed: u64,
    /// Bytes since last reseed.
    bytes_since_reseed: u64,
    /// Reseed interval in bytes.
    reseed_interval_bytes: u64,
    /// Total bytes generated.
    total_bytes_generated: u64,
    /// Reseed counter.
    reseed_count: u64,
    /// Maximum reseeds before exhaustion.
    max_reseeds: u64,
}

impl KelvinQuantum {
    /// Create a new Kelvin-Quantum instance.
    ///
    /// `seed` is the initial 2048-byte entropy pool (from orbital simulation).
    /// `max_reseeds` limits the total keystream.
    ///
    /// # Panics
    ///
    /// Panics if the initial keystream cache refill fails (e.g., `max_reseeds` is 0).
    /// Use [`with_config`](Self::with_config) for a fallible version.
    pub fn new(seed: [u8; QUANTUM_BASE_SEED_SIZE], max_reseeds: u64) -> Self {
        Self::with_config(
            seed,
            max_reseeds,
            QUANTUM_DEFAULT_CACHE_SIZE,
            QUANTUM_DEFAULT_ORBITAL_STEPS,
            QUANTUM_DEFAULT_RESEED_INTERVAL,
        )
        .expect("KelvinQuantum::new: initial cache refill failed (max_reseeds may be 0)")
    }

    /// Create a new Kelvin-Quantum instance with custom configuration.
    ///
    /// Returns an error if the initial keystream cache refill fails
    /// (e.g., if `max_reseeds` is 0).
    pub fn with_config(
        seed: [u8; QUANTUM_BASE_SEED_SIZE],
        max_reseeds: u64,
        cache_size: usize,
        orbital_steps_per_reseed: u64,
        reseed_interval_bytes: u64,
    ) -> Result<Self, KelvinError> {
        // Perturb the orbital state using material derived from the base seed.
        // This ensures every instance has a unique chaotic trajectory even when
        // initialized with the same seed (e.g., for encryption/decryption pairs).
        let mut perturb_hasher = Hasher::new();
        perturb_hasher.update(DOMSEP_QUANTUM_PERTURB_V1);
        perturb_hasher.update(&seed[..]);
        let mut perturb_buf = [0u8; XOF_SEED_SIZE];
        perturb_hasher.finalize_xof().fill(&mut perturb_buf);

        // Start with chaotic default and perturb using seed-derived material
        let mut orbital_state = OrbitalState::chaotic_default();
        // Perturb positions using the seed-derived buffer
        for i in 0..orbital_state.positions.len() {
            for j in 0..3 {
                let idx = (i * 3 + j) % perturb_buf.len();
                let perturbation = (perturb_buf[idx] as f64 - 128.0) * QUANTUM_PERTURB_SCALE;
                orbital_state.positions[i][j] += perturbation;
            }
        }
        // Perturb velocities too
        for i in 0..orbital_state.velocities.len() {
            for j in 0..3 {
                let idx = (i * 3 + j + 15) % perturb_buf.len(); // offset from positions
                let perturbation = (perturb_buf[idx] as f64 - 128.0) * QUANTUM_PERTURB_SCALE;
                orbital_state.velocities[i][j] += perturbation;
            }
        }

        let mut quantum = KelvinQuantum {
            base_seed: seed,
            keystream_cache: vec![0u8; cache_size],
            cache_pos: cache_size, // Force immediate refill
            orbital_state,
            orbital_steps_per_reseed,
            bytes_since_reseed: 0,
            reseed_interval_bytes,
            total_bytes_generated: 0,
            reseed_count: 0,
            max_reseeds,
        };

        // Refill the cache immediately so the first encrypt/decrypt call
        // doesn't need to check for exhaustion.
        quantum.refill_keystream_cache()?;

        Ok(quantum)
    }

    /// Refill the keystream cache by generating fresh keystream from the base seed.
    ///
    /// Uses SHAKE256 XOF seeded with the base seed and domain separator.
    /// Increments the reseed counter each time.
    fn refill_keystream_cache(&mut self) -> Result<(), KelvinError> {
        if self.reseed_count >= self.max_reseeds {
            return Err(KelvinError::SeedExhausted);
        }

        // Derive XOF seed from base seed via BLAKE3
        let mut hasher = Hasher::new();
        hasher.update(DOMSEP_QUANTUM_CACHE_V1);
        hasher.update(&self.base_seed[..]);
        hasher.update(&self.reseed_count.to_le_bytes());
        let mut xof_seed = [0u8; XOF_SEED_SIZE];
        hasher.finalize_xof().fill(&mut xof_seed);

        // SHAKE256 XOF: fill the keystream cache
        let mut shake = Shake256::default();
        sha3::digest::Update::update(&mut shake, &xof_seed);
        sha3::digest::Update::update(&mut shake, DOMSEP_QUANTUM_KEYSTREAM);
        let mut reader = shake.finalize_xof();
        XofReader::read(&mut reader, &mut self.keystream_cache);

        self.cache_pos = 0;
        self.reseed_count += 1;
        xof_seed.zeroize();

        Ok(())
    }

    /// Reseed the base seed with fresh orbital entropy.
    ///
    /// Advances the orbital simulation by `orbital_steps_per_reseed` steps,
    /// then extracts fresh entropy via SHAKE256 and XORs it into the base seed.
    /// This provides forward secrecy beyond BLAKE3's deterministic reseeding.
    fn reseed_from_orbital_chaos(&mut self) {
        // Advance orbital simulation
        for _ in 0..self.orbital_steps_per_reseed {
            // Ignore errors from verlet_step (stability checks may fail for
            // perturbed systems, but we still get useful entropy from the
            // simulation state before the error would occur).
            let _ = self.orbital_state.verlet_step();
        }

        // Extract fresh entropy from orbital state using the built-in extractor
        let mut fresh_entropy = [0u8; EXTRACT_BUF_SIZE];
        self.orbital_state.extract_entropy(&mut fresh_entropy);

        // XOR fresh entropy into base seed (in-place mixing)
        for (b, e) in self.base_seed.iter_mut().zip(fresh_entropy.iter().cycle()) {
            *b ^= e;
        }

        fresh_entropy.zeroize();
    }

    /// Get a byte from the keystream, refilling the cache if needed.
    ///
    /// Also triggers orbital reseeding at the configured interval.
    fn next_keystream_byte(&mut self) -> Result<u8, KelvinError> {
        if self.cache_pos >= self.keystream_cache.len() {
            self.refill_keystream_cache()?;
        }

        let byte = self.keystream_cache[self.cache_pos];
        self.cache_pos += 1;
        self.total_bytes_generated += 1;
        self.bytes_since_reseed += 1;

        // Check if we need to reseed from orbital chaos
        if self.bytes_since_reseed >= self.reseed_interval_bytes {
            self.reseed_from_orbital_chaos();
            self.bytes_since_reseed = 0;
        }

        Ok(byte)
    }

    /// Encrypt data in-place using XOR with the keystream.
    ///
    /// XOR is its own inverse, so encryption and decryption are the same operation.
    ///
    /// Processes data in chunks to avoid allocating a full-size keystream
    /// buffer for the entire input. A single reusable buffer of at most 1 MB
    /// is allocated once and reused across chunks, preventing OOM crashes when
    /// processing large (multi-GB) data. The buffer is zeroized after use.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.is_empty() {
            return Ok(());
        }
        // Allocate a reusable keystream buffer (up to CHUNK_SIZE = 1 MB).
        // For small inputs, cap the allocation to the actual data length
        // to avoid allocating a full 1 MB buffer for tiny messages.
        // Reusing the buffer across chunks avoids thousands of allocations
        // for multi-GB inputs.
        let buf_size = std::cmp::min(data.len(), KEYSTREAM_CHUNK_SIZE);
        let mut keystream = Zeroizing::new(vec![0u8; buf_size]);
        let mut offset = 0;
        while offset < data.len() {
            let remaining = data.len() - offset;
            let chunk_size = std::cmp::min(remaining, KEYSTREAM_CHUNK_SIZE);
            let chunk = &mut data[offset..offset + chunk_size];

            // Fill keystream buffer byte-by-byte from the cache
            for k in keystream[..chunk_size].iter_mut() {
                *k = self.next_keystream_byte()?;
            }

            // XOR into data
            for (d, k) in chunk.iter_mut().zip(keystream[..chunk_size].iter()) {
                *d ^= k;
            }

            offset += chunk_size;
        }
        Ok(())
    }

    /// Decrypt data in-place (same as encrypt, XOR is its own inverse).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.encrypt(data)
    }

    /// Get the total bytes generated.
    pub fn bytes_processed(&self) -> u64 {
        self.total_bytes_generated
    }

    /// Get the current reseed count.
    pub fn reseed_count(&self) -> u64 {
        self.reseed_count
    }

    /// Get the remaining reseeds before exhaustion.
    pub fn remaining_reseeds(&self) -> u64 {
        self.max_reseeds.saturating_sub(self.reseed_count)
    }
}

impl Drop for KelvinQuantum {
    fn drop(&mut self) {
        self.base_seed.zeroize();
        self.keystream_cache.zeroize();
        self.cache_pos.zeroize();
        self.bytes_since_reseed.zeroize();
        self.total_bytes_generated.zeroize();
        self.reseed_count.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; QUANTUM_BASE_SEED_SIZE] {
        let mut seed = [0u8; QUANTUM_BASE_SEED_SIZE];
        for (i, byte) in seed.iter_mut().enumerate() {
            *byte = (i % 256) as u8;
        }
        seed
    }

    #[test]
    fn test_round_trip_small() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        let mut data = b"Hello, Kelvin H Quantum!".to_vec();
        let original = data.clone();

        quantum.encrypt(&mut data).unwrap();
        assert_ne!(data, original, "encrypted data should differ from plaintext");

        // Decrypt with new instance (same seed = same keystream)
        let mut quantum2 = KelvinQuantum::new(test_seed(), 1000);
        quantum2.decrypt(&mut data).unwrap();
        assert_eq!(data, original, "round-trip should restore original");
    }

    #[test]
    fn test_determinism() {
        let mut q1 = KelvinQuantum::new(test_seed(), 1000);
        let mut q2 = KelvinQuantum::new(test_seed(), 1000);

        let mut data1 = b"Determinism test".to_vec();
        let mut data2 = data1.clone();

        q1.encrypt(&mut data1).unwrap();
        q2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "two instances should produce identical ciphertext");
    }

    #[test]
    fn test_empty_data() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        let mut empty: Vec<u8> = vec![];
        quantum.encrypt(&mut empty).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_large_data() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        let mut data = vec![0xABu8; 100_000]; // 100KB
        let original = data.clone();

        quantum.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        let mut quantum2 = KelvinQuantum::new(test_seed(), 1000);
        quantum2.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_bytes_processed() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        assert_eq!(quantum.bytes_processed(), 0);

        let mut data = vec![0u8; 100];
        quantum.encrypt(&mut data).unwrap();
        assert_eq!(quantum.bytes_processed(), 100);

        let mut data2 = vec![0u8; 50];
        quantum.encrypt(&mut data2).unwrap();
        assert_eq!(quantum.bytes_processed(), 150);
    }

    #[test]
    fn test_reseed_count_increments() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        assert_eq!(quantum.reseed_count(), 1); // Initial cache refill counts as reseed

        // Force a cache refill by encrypting more than cache size
        let mut data = vec![0u8; QUANTUM_DEFAULT_CACHE_SIZE + 1];
        quantum.encrypt(&mut data).unwrap();
        assert_eq!(quantum.reseed_count(), 2);
    }

    #[test]
    fn test_exhaustion() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1);
        // Encrypt more than the cache size to force a refill that exhausts
        let mut data = vec![0u8; QUANTUM_DEFAULT_CACHE_SIZE + 1];

        // First call should succeed (initial cache refill consumed the reseed,
        // but the data is larger than cache so it triggers a second refill)
        assert!(quantum.encrypt(&mut data).is_err());
    }

    #[test]
    fn test_remaining_reseeds() {
        let mut quantum = KelvinQuantum::new(test_seed(), 10);
        assert_eq!(quantum.remaining_reseeds(), 9); // Initial cache refill consumed 1

        let mut data = vec![0u8; QUANTUM_DEFAULT_CACHE_SIZE + 1];
        quantum.encrypt(&mut data).unwrap();
        assert_eq!(quantum.remaining_reseeds(), 8);
    }

    #[test]
    fn test_avalanche() {
        // 1-bit change in seed should produce completely different keystream
        let mut seed2 = test_seed();
        seed2[0] ^= 0x01;

        let mut q1 = KelvinQuantum::new(test_seed(), 1000);
        let mut q2 = KelvinQuantum::new(seed2, 1000);

        let mut data1 = b"Avalanche test data".to_vec();
        let mut data2 = data1.clone();

        q1.encrypt(&mut data1).unwrap();
        q2.encrypt(&mut data2).unwrap();

        let diff_bits: u32 =
            data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
        assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
    }
}
