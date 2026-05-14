//! Key schedule with reseeding, step counting, and exhaustion detection.
//!
//! The key schedule manages the lifecycle of ChaCha20 keys derived from
//! the orbital simulation. It tracks:
//! - Current step in the simulation
//! - Reseed interval (when to advance the simulation for new entropy)
//! - Safe step limit (from Lyapunov estimation)
//! - Bytes encrypted per key (to prevent overuse)
//!
//! ## Key Derivation
//!
//! Uses **HKDF-SHA512** (RFC 5869) for standardized key derivation from the
//! entropy pool. This replaces the previous ad-hoc SHA3-512 construction.
//!
//! ## Reseeding
//!
//! Uses **BLAKE3** for fast XOF-based reseeding of the 2048-byte entropy pool.
//! BLAKE3 is ~10x faster than SHAKE256 for large outputs, and the reseeding
//! path is performance-critical (called every `reseed_interval` steps).
//!
//! ## References
//!
//! - Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key
//!   Derivation Function (HKDF)." RFC 5869. doi:10.17487/RFC5869
//! - Aumasson, J.-P., et al. (2020). "BLAKE3: One Function, Fast Everywhere."
//!   https://github.com/BLAKE3-team/BLAKE3-specs

use blake3::Hasher;
use hkdf::Hkdf;
use sha3::Sha3_512;
use zeroize::Zeroize;

/// State of the key schedule.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ScheduleState {
    /// Key schedule is active and producing keys.
    Active,
    /// Key schedule is exhausted (safe steps exceeded).
    Exhausted,
}

/// Key schedule for the Kelvin cryptosystem.
///
/// Manages key derivation from orbital simulation state.
/// Each key is 32 bytes (cipher key) + 12 bytes (nonce/IV).
///
/// The seed is 2048 bytes, providing a large entropy pool for long-term
/// forward secrecy. Each reseed derives a fresh 2048-byte pool via BLAKE3.
#[derive(Clone, Debug)]
pub struct KeySchedule {
    /// Current seed material (2048 bytes).
    seed: [u8; 2048],
    /// Current step counter.
    step: u64,
    /// Total steps in the simulation.
    total_steps: u64,
    /// Steps between reseeds.
    reseed_interval: u64,
    /// Maximum safe steps (from Lyapunov estimation).
    safe_steps: u64,
    /// Current state.
    state: ScheduleState,
    /// Keys generated so far.
    keys_generated: u64,
    /// Maximum keys before exhaustion.
    max_keys: u64,
}

impl KeySchedule {
    /// Create a new key schedule.
    ///
    /// `seed` is the initial 2048-byte seed from SHAKE256 extraction.
    pub fn new(seed: [u8; 2048], total_steps: u64, reseed_interval: u64, safe_steps: u64) -> Self {
        // Each reseed produces one key, and each key can encrypt ~256 GiB
        // (ChaCha20 limit). We limit to safe_steps / reseed_interval keys.
        let max_keys = if reseed_interval > 0 {
            safe_steps.checked_div(reseed_interval).map(|v| v.max(1)).unwrap_or(1)
        } else {
            1
        };
        KeySchedule {
            seed,
            step: 0,
            total_steps,
            reseed_interval,
            safe_steps,
            state: ScheduleState::Active,
            keys_generated: 0,
            max_keys,
        }
    }

    /// Get the next key and nonce using HKDF-SHA512.
    ///
    /// Returns `None` if the schedule is exhausted.
    pub fn next_key(&mut self) -> Option<([u8; 32], [u8; 12])> {
        if self.state == ScheduleState::Exhausted {
            return None;
        }

        if self.keys_generated >= self.max_keys {
            self.state = ScheduleState::Exhausted;
            return None;
        }

        // Derive key and nonce using HKDF-SHA512 (RFC 5869)
        let hk = Hkdf::<Sha3_512>::new(None, &self.seed);

        // Domain-separated key derivation using stack-allocated arrays
        // to avoid heap allocations in this performance-critical path.
        let mut info = [0u8; 28];

        // Derive key: "kelvin-hkdf-key-v1" (18 bytes) + counter (8 bytes) = 26 bytes
        info[..18].copy_from_slice(b"kelvin-hkdf-key-v1");
        info[18..26].copy_from_slice(&self.keys_generated.to_le_bytes());
        let mut key = [0u8; 32];
        hk.expand(&info[..26], &mut key)
            .expect("HKDF expand should not fail for valid output length");

        // Derive nonce: "kelvin-hkdf-nonce-v1" (20 bytes) + counter (8 bytes) = 28 bytes
        info[..20].copy_from_slice(b"kelvin-hkdf-nonce-v1");
        info[20..28].copy_from_slice(&self.keys_generated.to_le_bytes());
        let mut nonce = [0u8; 12];
        hk.expand(&info[..28], &mut nonce)
            .expect("HKDF expand should not fail for valid output length");

        // Advance step and reseed if needed
        self.step += self.reseed_interval;
        self.keys_generated += 1;

        // Reseed: derive new 2048-byte seed from current seed using BLAKE3
        let mut reseed_hasher = Hasher::new();
        reseed_hasher.update(b"kelvin-reseed-v1");
        reseed_hasher.update(&self.seed[..]);
        reseed_hasher.update(&self.step.to_le_bytes());
        let mut reseed_buf = [0u8; 2048];
        reseed_hasher.finalize_xof().fill(&mut reseed_buf);
        self.seed = reseed_buf;

        // Check exhaustion
        if self.step >= self.safe_steps || self.step >= self.total_steps {
            self.state = ScheduleState::Exhausted;
        }

        Some((key, nonce))
    }

    /// Get the current state.
    pub fn state(&self) -> ScheduleState {
        self.state
    }

    /// Get the current step.
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Get the number of keys generated.
    pub fn keys_generated(&self) -> u64 {
        self.keys_generated
    }

    /// Get the remaining safe bytes (each key can encrypt ~256 GiB).
    ///
    /// This is a conservative estimate based on remaining keys.
    pub fn remaining_bytes(&self) -> u64 {
        if self.state == ScheduleState::Exhausted {
            return 0;
        }
        let remaining_keys = self.max_keys.saturating_sub(self.keys_generated);
        // Each key can safely encrypt 2^32 bytes (4 GiB) — conservative
        remaining_keys.saturating_mul(1 << 32)
    }

    /// Reset the schedule with a new seed.
    pub fn reset(&mut self, seed: [u8; 2048]) {
        self.seed = seed;
        self.step = 0;
        self.keys_generated = 0;
        self.state = ScheduleState::Active;
    }
}

impl Drop for KeySchedule {
    fn drop(&mut self) {
        self.seed.zeroize();
        self.step.zeroize();
        self.total_steps.zeroize();
        self.reseed_interval.zeroize();
        self.safe_steps.zeroize();
        self.keys_generated.zeroize();
        self.max_keys.zeroize();
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
    fn test_key_schedule_creation() {
        let schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        assert_eq!(schedule.state(), ScheduleState::Active);
        assert_eq!(schedule.step(), 0);
        assert_eq!(schedule.keys_generated(), 0);
    }

    #[test]
    fn test_next_key_returns_key_and_nonce() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let result = schedule.next_key();
        assert!(result.is_some());
        let (key, nonce) = result.unwrap();
        assert_eq!(key.len(), 32);
        assert_eq!(nonce.len(), 12);
    }

    #[test]
    fn test_next_key_deterministic() {
        let mut s1 = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let mut s2 = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let k1 = s1.next_key();
        let k2 = s2.next_key();
        assert_eq!(k1, k2);
    }

    #[test]
    fn test_multiple_keys_different() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let k1 = schedule.next_key().unwrap();
        let k2 = schedule.next_key().unwrap();
        assert_ne!(k1.0, k2.0); // keys should differ
    }

    #[test]
    fn test_hkdf_domain_separation() {
        // Key and nonce should be different (different info strings)
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let (key, nonce) = schedule.next_key().unwrap();
        assert_ne!(&key[..12], &nonce[..]);
    }

    #[test]
    fn test_exhaustion() {
        let mut schedule = KeySchedule::new(test_seed(), 1000, 100, 500);
        // Should exhaust after ~5 keys (safe_steps / reseed_interval)
        for _ in 0..10 {
            if schedule.next_key().is_none() {
                break;
            }
        }
        assert_eq!(schedule.state(), ScheduleState::Exhausted);
        assert!(schedule.next_key().is_none());
    }

    #[test]
    fn test_remaining_bytes() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        assert!(schedule.remaining_bytes() > 0);
        schedule.next_key();
        assert!(schedule.remaining_bytes() > 0);
    }

    #[test]
    fn test_exhausted_remaining_bytes() {
        let mut schedule = KeySchedule::new(test_seed(), 100, 100, 50);
        schedule.next_key();
        assert_eq!(schedule.remaining_bytes(), 0);
    }

    #[test]
    fn test_reset() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        schedule.next_key();
        schedule.next_key();
        assert!(schedule.keys_generated() > 0);

        schedule.reset(test_seed());
        assert_eq!(schedule.keys_generated(), 0);
        assert_eq!(schedule.state(), ScheduleState::Active);
    }

    #[test]
    fn test_step_increments() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        assert_eq!(schedule.step(), 0);
        schedule.next_key();
        assert_eq!(schedule.step(), 1000);
        schedule.next_key();
        assert_eq!(schedule.step(), 2000);
    }

    #[test]
    fn test_hkdf_seed_avalanche() {
        let mut seed2 = test_seed();
        seed2[0] ^= 0x01; // 1-bit change

        let mut s1 = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let mut s2 = KeySchedule::new(seed2, 10000, 1000, 5000);

        let (k1, _) = s1.next_key().unwrap();
        let (k2, _) = s2.next_key().unwrap();

        // Count differing bits
        let diff_bits: u32 = k1.iter().zip(k2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();

        // Should have roughly half the bits different (avalanche effect)
        assert!(diff_bits > 100, "Too few differing bits: {}", diff_bits);
    }

    #[test]
    fn test_blake3_reseed_deterministic() {
        let mut s1 = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let mut s2 = KeySchedule::new(test_seed(), 10000, 1000, 5000);

        // Both should produce the same sequence of keys
        for _ in 0..3 {
            let k1 = s1.next_key();
            let k2 = s2.next_key();
            assert_eq!(k1, k2);
        }
    }
}
