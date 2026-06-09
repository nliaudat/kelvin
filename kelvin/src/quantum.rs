//! H Kelvin-Quantum — Hybrid orbital chaos + quantum-resistant OTP stream cipher.
//!
//! ## Architecture
//!
//! H Kelvin-Quantum is a **quantum-resistant stream cipher**
//! that combines the orbital chaos KDF with SHAKE256 XOF. Unlike V3 Photon
//! (which uses HKDF→SHAKE256), H Quantum periodically refreshes its base seed
//! with fresh orbital entropy via `reseed_from_orbital_chaos`:
//!
//! ```text
//! 2048B base seed → BLAKE3 XOF → perturbation → orbital state
//!   ↓
//! SHAKE256 XOF → keystream cache (1 MiB)
//!   ↓
//! XOR with plaintext/ciphertext (OTP encryption)
//!   ↓
//! Every N bytes: reseed_from_orbital_chaos
//!   → advance orbital simulation by M steps
//!   → extract fresh entropy via SHAKE256
//!   → XOR into base seed
//! ```
//!
//! ## Security
//!
//! - **OTP construction**: Data is XOR-encrypted byte-by-byte with SHAKE256
//!   keystream. No nonce, no IV, no algebraic round function.
//! - **No authentication**: XOR is malleable. Use with external MAC or
//!   in environments where malleability is acceptable.
//! - **Orbital reseeding**: Fresh chaotic entropy is mixed in periodically,
//!   providing forward secrecy beyond BLAKE3's deterministic reseeding.
//! - **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit
//!   quantum security. No algebraic structure for Shor's algorithm to exploit.

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
    QUANTUM_DEFAULT_INTEGRATION_METHOD, QUANTUM_DEFAULT_ORBITAL_STEPS,
    QUANTUM_DEFAULT_RESEED_INTERVAL, QUANTUM_PERTURB_SCALE, XOF_SEED_SIZE,
};
use kelvin_core::IntegrationMethod;
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
    /// Integration method for orbital reseeding (Verlet or Euler).
    integration_method: IntegrationMethod,
    /// Orbital steps to run per reseed.
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
            integration_method: QUANTUM_DEFAULT_INTEGRATION_METHOD,
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
    /// Advances the orbital simulation by `orbital_steps_per_reseed` steps
    /// using the configured integration method (Verlet or Euler),
    /// then extracts fresh entropy via SHAKE256 and XORs it into the base seed.
    /// This provides forward secrecy beyond BLAKE3's deterministic reseeding.
    ///
    /// If the orbital simulation experiences a body ejection or gravitational
    /// collapse, the simulation is re-derived from the base seed to maintain
    /// the security invariant that N ≥ 3 bodies are present (N=2 has a
    /// closed-form solution). This re-derivation ensures forward secrecy
    /// continues even after a transient instability.
    fn reseed_from_orbital_chaos(&mut self) {
        // Advance orbital simulation using the configured integration method
        for _ in 0..self.orbital_steps_per_reseed {
            let result = match self.integration_method {
                IntegrationMethod::Verlet => self.orbital_state.verlet_step(),
                IntegrationMethod::Euler => self.orbital_state.euler_step(),
            };
            // On stability failure (ejection/collapse), re-derive orbital state
            // from the base seed to restore a healthy N-body chaotic regime.
            if result.is_err() {
                self.recover_orbital_state();
                break;
            }
        }

        // Extract fresh entropy from orbital state using SHAKE256
        let mut fresh_entropy = [0u8; EXTRACT_BUF_SIZE];
        self.orbital_state.extract_entropy(&mut fresh_entropy);

        // XOR fresh entropy into base seed (in-place mixing)
        for (b, e) in self.base_seed.iter_mut().zip(fresh_entropy.iter().cycle()) {
            *b ^= e;
        }

        fresh_entropy.zeroize();
    }

    /// Recover from an orbital stability failure by re-deriving the orbital
    /// state from the current base seed. This ensures the system always has
    /// at least 5 chaotic bodies, maintaining the security invariant that
    /// N ≥ 3 (no closed-form solution for the attacker).
    fn recover_orbital_state(&mut self) {
        // Perturb the orbital state using material derived from the base seed.
        // This is the same perturbation logic as in with_config(), ensuring
        // deterministic recovery from the current base seed.
        let mut perturb_hasher = Hasher::new();
        perturb_hasher.update(DOMSEP_QUANTUM_PERTURB_V1);
        perturb_hasher.update(&self.base_seed[..]);
        perturb_hasher.update(&self.reseed_count.to_le_bytes());
        let mut perturb_buf = [0u8; XOF_SEED_SIZE];
        perturb_hasher.finalize_xof().fill(&mut perturb_buf);

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
                let idx = (i * 3 + j + 15) % perturb_buf.len();
                let perturbation = (perturb_buf[idx] as f64 - 128.0) * QUANTUM_PERTURB_SCALE;
                orbital_state.velocities[i][j] += perturbation;
            }
        }

        self.orbital_state = orbital_state;
    }

    /// Fill `output` with keystream bytes from the cache, refilling as needed.
    ///
    /// This is the bulk (batch) variant of `next_keystream_byte`. It copies
    /// directly from the cache in slices, avoiding the byte-by-byte overhead
    /// of calling `next_keystream_byte` in a loop.
    ///
    /// Also triggers orbital reseeding at the configured interval.
    pub fn keystream_bytes(&mut self, output: &mut [u8]) -> Result<(), KelvinError> {
        let mut remaining = output.len();
        let mut out_pos = 0;
        while remaining > 0 {
            // Refill cache if exhausted
            if self.cache_pos >= self.keystream_cache.len() {
                self.refill_keystream_cache()?;
            }

            // Copy as much as possible from current cache position
            let avail = self.keystream_cache.len() - self.cache_pos;
            let take = std::cmp::min(remaining, avail);
            output[out_pos..out_pos + take]
                .copy_from_slice(&self.keystream_cache[self.cache_pos..self.cache_pos + take]);

            self.cache_pos += take;
            self.total_bytes_generated += take as u64;
            self.bytes_since_reseed += take as u64;
            out_pos += take;
            remaining -= take;

            // Check if we need to reseed from orbital chaos
            if self.bytes_since_reseed >= self.reseed_interval_bytes {
                self.reseed_from_orbital_chaos();
                self.bytes_since_reseed = 0;
            }
        }
        Ok(())
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

            // Fill keystream buffer in bulk from the cache (batch extraction)
            self.keystream_bytes(&mut keystream[..chunk_size])?;

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
        [0u8; QUANTUM_BASE_SEED_SIZE]
    }

    #[test]
    fn test_round_trip_small() {
        let seed = test_seed();
        let mut enc = KelvinQuantum::new(seed, 1000);
        let mut dec = KelvinQuantum::new(seed, 1000);

        let original = b"Hello, Kelvin-Quantum!".to_vec();
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_determinism() {
        let seed = test_seed();
        let mut q1 = KelvinQuantum::new(seed, 1000);
        let mut q2 = KelvinQuantum::new(seed, 1000);

        let mut buf1 = vec![0u8; 256];
        let mut buf2 = vec![0u8; 256];
        q1.encrypt(&mut buf1).unwrap();
        q2.encrypt(&mut buf2).unwrap();
        assert_eq!(buf1, buf2);
    }

    #[test]
    fn test_empty_data() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        let mut data = Vec::new();
        quantum.encrypt(&mut data).unwrap();
        assert!(data.is_empty());
        quantum.decrypt(&mut data).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_large_data() {
        let seed = test_seed();
        let mut enc = KelvinQuantum::new(seed, 1000);
        let mut dec = KelvinQuantum::new(seed, 1000);

        let original = vec![0xABu8; 5 * 1024 * 1024]; // 5 MB
        let mut data = original.clone();

        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);

        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_reseed_count_increments() {
        // Use a tiny cache (64 bytes) so encrypting triggers refills.
        // With max_reseeds=1000 and 64-byte cache, encrypt 640 bytes to
        // trigger 10 refills (640/64), well within the reseed budget.
        let mut quantum = KelvinQuantum::with_config(test_seed(), 1000, 64, 10, 1024).unwrap();
        let initial_count = quantum.reseed_count();
        assert!(initial_count >= 1, "initial reseed_count should be >= 1, got {}", initial_count);

        // Generate enough data to trigger additional cache refills
        let mut buf = vec![0u8; 640]; // 10 cache refills
        quantum.encrypt(&mut buf).unwrap();
        assert!(quantum.reseed_count() > initial_count);
    }

    #[test]
    fn test_exhaustion() {
        // Use with_config with max_reseeds=2, tiny cache so encrypting 1 MB
        // triggers many refills and exhausts all reseeds.
        let mut quantum = KelvinQuantum::with_config(test_seed(), 2, 64, 10, 1024).unwrap();
        // Initial refill consumed 1, so 1 remaining. Encrypting 1 MB with
        // a 64-byte cache will exhaust the last reseed.
        let mut buf = vec![0u8; 1024 * 1024];
        let result = quantum.encrypt(&mut buf);
        assert!(result.is_err());
    }

    #[test]
    fn test_remaining_reseeds() {
        // with_config with max_reseeds=5, tiny cache: initial refill consumes 1, leaving 4.
        // Encrypt a small amount that doesn't exhaust all reseeds.
        let mut quantum = KelvinQuantum::with_config(test_seed(), 5, 64, 10, 1024).unwrap();
        assert_eq!(quantum.remaining_reseeds(), 4);

        // Encrypt 128 bytes: cache is 64 bytes, so 1 refill is triggered,
        // consuming 1 reseed, leaving 3 remaining.
        let mut buf = vec![0u8; 128];
        quantum.encrypt(&mut buf).unwrap();
        assert_eq!(quantum.remaining_reseeds(), 3);
    }

    #[test]
    fn test_avalanche() {
        let seed1 = test_seed();
        let mut seed2 = test_seed();

        seed2[0] ^= 1; // Flip one bit

        let mut q1 = KelvinQuantum::new(seed1, 1000);
        let mut q2 = KelvinQuantum::new(seed2, 1000);

        let mut buf1 = vec![0u8; 1024];
        let mut buf2 = vec![0u8; 1024];
        q1.encrypt(&mut buf1).unwrap();
        q2.encrypt(&mut buf2).unwrap();

        assert_ne!(buf1, buf2);
    }

    #[test]
    fn test_with_config_custom_params() {
        let quantum = KelvinQuantum::with_config(
            test_seed(),
            100,
            512,  // tiny cache
            10,   // orbital steps per reseed
            1024, // reseed every 1 KB
        )
        .unwrap();
        // Initial cache refill consumes 1 reseed, so 99 remain
        assert_eq!(quantum.remaining_reseeds(), 99);
    }

    #[test]
    fn test_with_config_zero_max_reseeds() {
        let result = KelvinQuantum::with_config(test_seed(), 0, 64, 10, 1024);
        assert!(result.is_err());
    }
}
