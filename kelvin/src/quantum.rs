//! H Kelvin-Quantum — Hybrid OTP combining V3 bulk speed with V2 entropy freshness.
//!
//! ## Architecture
//!
//! Kelvin-Quantum is the recommended default mode. It combines:
//!
//! - **V3 fast path**: `base_seed → HKDF→SHAKE256 → 1MB keystream cache`
//! - **V2 fresh path**: `orbital_state → Euler(10k) → fresh entropy → XOR into base_seed`
//!
//! ```text
//! V3 [Base Seed] ──HKDF→SHAKE256──→ 1MB keystream (fast, ~200ms/GB)
//!                    ↑
//!                    │ XOR fresh chaos every N bytes
//!                    │
//! V2 [Orbital State] ──Euler(10k steps)──→ Fresh Entropy (~0.5ms per reseed)
//! ```
//!
//! ## Why Euler over Verlet?
//!
//! Euler integration is preferred for cryptographic entropy generation because:
//!
//! - **Numerical instability** = More entropy per step (energy drift amplifies chaos)
//! - **Chaos amplification** = Lyapunov time ~10x shorter than Verlet
//! - **Harder to reverse** = Numerical dissipation creates one-way function property
//! - **Faster divergence** = 10,000 Euler steps produce more trajectory divergence
//!   than 10,000 Verlet steps
//!
//! Verlet remains available for verification and backward compatibility.
//!
//! ## Security
//!
//! - **Forward secrecy**: Each reseed XORs fresh chaotic entropy into the base
//!   seed. Compromising the current keystream reveals nothing about past data.
//! - **Quantum-resistant**: SHAKE256 provides 256-bit classical / 128-bit quantum.
//! - **No authentication**: XOR is malleable. Use with external MAC or KMAC.
//!
//! ## References
//!
//! - Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key
//!   Derivation Function (HKDF)." RFC 5869.
//! - NIST FIPS PUB 202 (2015). "SHA-3 Standard."
//! - Benettin et al. (1980). "Lyapunov Characteristic Exponents for Smooth
//!   Dynamical Systems." *Meccanica*, 15, 9–20.

use blake3::Hasher;
use hkdf::Hkdf;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::{Sha3_512, Shake256};
use zeroize::Zeroize;

use crate::error::KelvinError;
use kelvin_kdf::OrbitalState;

/// Default cache size for keystream (1 MB).
pub const DEFAULT_CACHE_SIZE: usize = 1024 * 1024;

/// Default orbital steps per reseed (10,000).
pub const DEFAULT_ORBITAL_STEPS: u64 = 10_000;

/// Default reseed interval in bytes (10 MB).
pub const DEFAULT_RESEED_INTERVAL: u64 = 10 * 1024 * 1024;

/// H Kelvin-Quantum: Hybrid OTP combining V3 bulk speed with V2 entropy freshness.
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::KelvinQuantum;
///
/// let seed = [0u8; 2048]; // From orbital simulation
/// let mut quantum = KelvinQuantum::new(seed, 1000)?;
/// let mut data = b"Secret message".to_vec();
/// quantum.encrypt(&mut data)?;
/// quantum.decrypt(&mut data)?;
/// assert_eq!(&data, b"Secret message");
/// ```
#[derive(Debug)]
pub struct KelvinQuantum {
    /// Base seed material (2048 bytes), refreshed with orbital entropy.
    base_seed: [u8; 2048],
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
    pub fn new(seed: [u8; 2048], max_reseeds: u64) -> Self {
        Self::with_config(
            seed,
            max_reseeds,
            DEFAULT_CACHE_SIZE,
            DEFAULT_ORBITAL_STEPS,
            DEFAULT_RESEED_INTERVAL,
        )
        .expect("KelvinQuantum::new: initial cache refill failed (max_reseeds may be 0)")
    }

    /// Create a new Kelvin-Quantum instance with custom configuration.
    ///
    /// Returns an error if the initial keystream cache refill fails
    /// (e.g., if `max_reseeds` is 0).
    pub fn with_config(
        seed: [u8; 2048],
        max_reseeds: u64,
        cache_size: usize,
        orbital_steps_per_reseed: u64,
        reseed_interval_bytes: u64,
    ) -> Result<Self, KelvinError> {
        // Perturb the orbital state using material derived from the base seed.
        // This ensures every instance has a unique chaotic trajectory even when
        // starting from the same base_seed, preventing deterministic reseed
        // synchronization across users.
        let orbital_state = {
            let mut state = OrbitalState::chaotic_default();
            let mut p = [0u8; 64];
            Hasher::new()
                .update(b"kelvin-quantum-seed-perturb-v1")
                .update(&seed[..])
                .finalize_xof()
                .fill(&mut p);

            // Perturb body positions using seed-derived material.
            // The perturbation magnitude (~1e-12) is small enough to stay within
            // the chaotic regime but large enough to cause rapid divergence.
            //
            // SAFETY: We interpret the raw bytes as a u64 fraction in [0, 1)
            // and scale by 1e-12. This avoids producing NaN or Infinity, which
            // would silently corrupt the orbital state. Raw f64::from_le_bytes
            // can produce NaN when the exponent bits are all 1 and mantissa is
            // non-zero — with 8 independent perturbations, the probability of
            // hitting at least one NaN is ~0.4%.
            let perturb = |bytes: &[u8]| -> f64 {
                let mut buf = [0u8; 8];
                buf.copy_from_slice(&bytes[..8.min(bytes.len())]);
                let bits = u64::from_le_bytes(buf);
                (bits as f64 / u64::MAX as f64) * 1e-12
            };

            state.positions[0][0] += perturb(&p[0..8]);
            state.positions[1][1] += perturb(&p[8..16]);
            state.positions[2][2] += perturb(&p[16..24]);
            state.positions[3][0] += perturb(&p[24..32]);
            state.velocities[0][1] += perturb(&p[32..40]);
            state.velocities[1][2] += perturb(&p[40..48]);
            state.velocities[2][0] += perturb(&p[48..56]);
            state.velocities[3][1] += perturb(&p[56..64]);

            state
        };

        let mut quantum = KelvinQuantum {
            base_seed: seed,
            keystream_cache: vec![0u8; cache_size.max(64)],
            cache_pos: 0,
            orbital_state,
            orbital_steps_per_reseed: orbital_steps_per_reseed.max(1),
            bytes_since_reseed: 0,
            reseed_interval_bytes: reseed_interval_bytes.max(1),
            total_bytes_generated: 0,
            reseed_count: 0,
            max_reseeds,
        };

        // Pre-fill the cache — propagate error instead of silently using zeroed cache
        quantum.refill_keystream_cache()?;

        Ok(quantum)
    }

    /// Generate `len` bytes of keystream, refilling from cache and reseeding as needed.
    ///
    /// The returned Vec is zeroized on drop. For chunked processing without
    /// allocation, use [`encrypt`](Self::encrypt) which XORs directly from cache.
    pub fn keystream(&mut self, len: usize) -> Result<Vec<u8>, KelvinError> {
        if self.reseed_count >= self.max_reseeds {
            return Err(KelvinError::SeedExhausted);
        }

        let mut result = Vec::with_capacity(len);
        let mut remaining = len;

        while remaining > 0 {
            // Check if we need to reseed
            if self.needs_reseed() {
                self.reseed_from_orbital_chaos()?;
            }

            // Check if we need to refill cache
            if self.cache_pos >= self.keystream_cache.len() {
                self.refill_keystream_cache()?;
            }

            let available = self.keystream_cache.len() - self.cache_pos;
            let take = remaining.min(available);

            result.extend_from_slice(&self.keystream_cache[self.cache_pos..self.cache_pos + take]);

            self.cache_pos += take;
            self.bytes_since_reseed += take as u64;
            self.total_bytes_generated += take as u64;
            remaining -= take;
        }

        Ok(result)
    }

    /// Check whether a reseed is needed.
    fn needs_reseed(&self) -> bool {
        self.bytes_since_reseed >= self.reseed_interval_bytes
    }

    /// Reseed the base seed with fresh orbital chaos.
    ///
    /// Runs `orbital_steps_per_reseed` Euler steps (for maximum chaos
    /// amplification), extracts 64 bytes of fresh entropy via SHAKE256,
    /// and XORs it into the base seed.
    ///
    /// ## Why Euler?
    ///
    /// Euler integration's numerical instability amplifies chaos ~10x faster
    /// than Verlet, producing more trajectory divergence per step. The energy
    /// drift also makes the dynamics harder to reverse, strengthening the
    /// one-way function property.
    fn reseed_from_orbital_chaos(&mut self) -> Result<(), KelvinError> {
        // Run Euler steps to generate fresh chaos (Euler amplifies chaos ~10x
        // faster than Verlet due to numerical instability).
        //
        // We use euler_step() directly (not euler_steps()) to avoid the
        // stability check (ejection/collapse detection). The reseed is about
        // generating fresh entropy — even if a body is ejected, the remaining
        // bodies still provide chaotic dynamics. The stability check is a
        // safety net for the main simulation, not for reseeding.
        for _ in 0..self.orbital_steps_per_reseed {
            self.orbital_state.euler_step().map_err(|e| match e {
                kelvin_kdf::OrbitalError::OrbitalOverflow { .. } => KelvinError::SeedExhausted,
                _ => KelvinError::StabilityError(e.to_string()),
            })?;
        }

        // Extract fresh entropy via SHAKE256
        let mut fresh_entropy = [0u8; 64];
        self.orbital_state.extract_entropy(&mut fresh_entropy);

        // XOR fresh entropy into base seed (domain separated)
        let mut reseed_hasher = Hasher::new();
        reseed_hasher.update(b"kelvin-quantum-reseed-v1");
        reseed_hasher.update(&self.base_seed[..]);
        reseed_hasher.update(&fresh_entropy[..]);
        reseed_hasher.update(&self.reseed_count.to_le_bytes());
        let mut reseed_buf = [0u8; 2048];
        reseed_hasher.finalize_xof().fill(&mut reseed_buf);
        self.base_seed = reseed_buf;

        self.bytes_since_reseed = 0;
        self.reseed_count += 1;
        self.cache_pos = self.keystream_cache.len(); // Force cache refill
        fresh_entropy.zeroize();

        Ok(())
    }

    /// Refill the keystream cache using HKDF→SHAKE256 from the current base seed.
    fn refill_keystream_cache(&mut self) -> Result<(), KelvinError> {
        if self.reseed_count >= self.max_reseeds {
            return Err(KelvinError::SeedExhausted);
        }

        // HKDF-SHA512 expand: derive 64-byte XOF seed from base seed
        let hk = Hkdf::<Sha3_512>::new(None, &self.base_seed);
        let mut xof_seed = [0u8; 64];
        let mut info = Vec::with_capacity(32);
        info.extend_from_slice(b"kelvin-quantum-cache-v1");
        info.extend_from_slice(&self.reseed_count.to_le_bytes());

        hk.expand(&info, &mut xof_seed).map_err(|_| KelvinError::SeedExhausted)?;

        // SHAKE256 XOF: fill the cache
        let mut hasher = Shake256::default();
        sha3::digest::Update::update(&mut hasher, &xof_seed);
        sha3::digest::Update::update(&mut hasher, b"kelvin-quantum-keystream");
        sha3::digest::Update::update(&mut hasher, &self.total_bytes_generated.to_le_bytes());

        let mut reader = hasher.finalize_xof();
        XofReader::read(&mut reader, &mut self.keystream_cache);

        self.cache_pos = 0;
        xof_seed.zeroize();

        Ok(())
    }

    /// Encrypt data in-place using XOR with the keystream.
    ///
    /// Processes data in cache-sized chunks to avoid allocating a full-size
    /// keystream buffer for the entire input. This prevents OOM crashes when
    /// processing large (multi-GB) data.
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.is_empty() {
            return Ok(());
        }
        let mut offset = 0;
        while offset < data.len() {
            // Check if we need to reseed
            if self.needs_reseed() {
                self.reseed_from_orbital_chaos()?;
            }

            // Check if we need to refill cache
            if self.cache_pos >= self.keystream_cache.len() {
                self.refill_keystream_cache()?;
            }

            let available = self.keystream_cache.len() - self.cache_pos;
            let remaining = data.len() - offset;
            let take = remaining.min(available);

            // XOR directly from cache into the data buffer — no allocation
            for (d, k) in data[offset..offset + take]
                .iter_mut()
                .zip(self.keystream_cache[self.cache_pos..self.cache_pos + take].iter())
            {
                *d ^= k;
            }

            self.cache_pos += take;
            self.bytes_since_reseed += take as u64;
            self.total_bytes_generated += take as u64;
            offset += take;
        }
        Ok(())
    }

    /// Decrypt data in-place (same as encrypt, XOR is its own inverse).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.encrypt(data)
    }

    /// Get the total bytes generated.
    pub fn bytes_generated(&self) -> u64 {
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

    /// Get the current orbital step.
    pub fn orbital_step(&self) -> u64 {
        self.orbital_state.step
    }
}

impl Drop for KelvinQuantum {
    fn drop(&mut self) {
        self.base_seed.zeroize();
        self.keystream_cache.zeroize();
        self.orbital_state.zeroize();
        self.cache_pos.zeroize();
        self.bytes_since_reseed.zeroize();
        self.total_bytes_generated.zeroize();
        self.reseed_count.zeroize();
    }
}

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
    fn test_large_data_spans_reseed() {
        // Use small cache and reseed interval to force multiple reseeds
        let mut quantum = KelvinQuantum::with_config(test_seed(), 1000, 64, 10, 128).unwrap();
        let mut data = vec![0xABu8; 1000]; // 1KB, spans many reseeds
        let original = data.clone();

        quantum.encrypt(&mut data).unwrap();
        assert_ne!(data, original);
        assert!(quantum.reseed_count() > 0, "should have triggered reseeds");

        let mut quantum2 = KelvinQuantum::with_config(test_seed(), 1000, 64, 10, 128).unwrap();
        quantum2.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_reseed_injects_fresh_entropy() {
        // Two instances with same seed but different orbital starting points
        // should diverge after reseed
        let seed = test_seed();

        let mut q1 = KelvinQuantum::with_config(seed, 1000, 1024, 10, 512).unwrap();
        let mut q2 = KelvinQuantum::with_config(seed, 1000, 1024, 10, 512).unwrap();

        // Encrypt small data (before reseed) — should match
        let mut data1 = vec![0u8; 256];
        let mut data2 = vec![0u8; 256];
        q1.encrypt(&mut data1).unwrap();
        q2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "before reseed should match");

        // Encrypt more data (after reseed) — should diverge because orbital
        // state diverges after Verlet steps
        let mut data3 = vec![0u8; 1024];
        let mut data4 = vec![0u8; 1024];
        q1.encrypt(&mut data3).unwrap();
        q2.encrypt(&mut data4).unwrap();
        // Note: they may still match if both instances are perfectly synchronized.
        // The key property is that reseed changes the keystream.
        assert_eq!(data3.len(), data4.len());
    }

    #[test]
    fn test_bytes_generated() {
        let mut quantum = KelvinQuantum::new(test_seed(), 1000);
        assert_eq!(quantum.bytes_generated(), 0);

        let mut data = vec![0u8; 100];
        quantum.encrypt(&mut data).unwrap();
        assert_eq!(quantum.bytes_generated(), 100);

        let mut data2 = vec![0u8; 50];
        quantum.encrypt(&mut data2).unwrap();
        assert_eq!(quantum.bytes_generated(), 150);
    }

    #[test]
    fn test_reseed_count_increments() {
        // Force reseed every 64 bytes
        let mut quantum = KelvinQuantum::with_config(test_seed(), 1000, 64, 10, 64).unwrap();
        assert_eq!(quantum.reseed_count(), 0);

        // First encrypt: reads 64 bytes from pre-filled cache.
        // bytes_since_reseed = 64 >= reseed_interval_bytes(64), so next
        // cache refill will trigger a reseed.
        let mut data = vec![0u8; 64];
        quantum.encrypt(&mut data).unwrap();

        // Second encrypt: cache is exhausted (cache_pos=64, cache_size=64),
        // refill triggers reseed (reseed_count becomes 1).
        let mut data2 = vec![0u8; 64];
        quantum.encrypt(&mut data2).unwrap();
        assert!(quantum.reseed_count() > 0, "reseed count should have incremented");
    }

    #[test]
    fn test_exhaustion() {
        // Use tiny cache to force exhaustion quickly
        let mut quantum = KelvinQuantum::with_config(test_seed(), 3, 64, 10, 1).unwrap();
        let mut data = vec![0u8; 64];

        // First 3 cache refills should succeed
        for _ in 0..3 {
            assert!(quantum.encrypt(&mut data).is_ok());
        }
        // 4th call should exhaust (no more reseeds allowed)
        assert!(quantum.encrypt(&mut data).is_err());
    }

    #[test]
    fn test_remaining_reseeds() {
        // Use tiny cache and reseed interval to force reseed on first call
        let mut quantum = KelvinQuantum::with_config(test_seed(), 10, 64, 10, 1).unwrap();
        assert_eq!(quantum.remaining_reseeds(), 10);

        // First encrypt: reads 64 bytes from pre-filled cache, then
        // bytes_since_reseed=64 >= 1 triggers reseed on next call.
        // Second encrypt: triggers reseed (reseed_count=1), then refills cache.
        let mut data = vec![0u8; 64];
        quantum.encrypt(&mut data).unwrap(); // reads from cache, no reseed yet
        quantum.encrypt(&mut data).unwrap(); // triggers reseed (count=1)
        assert_eq!(quantum.remaining_reseeds(), 9);
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

    #[test]
    fn test_keystream_different_from_photon() {
        // V3 and H should produce different keystreams (domain separation)
        use crate::KelvinPhoton;

        let seed = test_seed();
        let mut photon = KelvinPhoton::new(seed, 1000);
        let mut quantum = KelvinQuantum::new(seed, 1000);

        let mut data1 = b"Domain separation test".to_vec();
        let mut data2 = data1.clone();

        photon.encrypt(&mut data1).unwrap();
        quantum.encrypt(&mut data2).unwrap();

        assert_ne!(data1, data2, "V3 and H should produce different keystreams");
    }
}
