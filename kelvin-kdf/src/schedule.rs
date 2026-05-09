//! Key schedule with reseeding, step counting, and exhaustion detection.
//!
//! The key schedule manages the lifecycle of ChaCha20 keys derived from
//! the orbital simulation. It tracks:
//! - Current step in the simulation
//! - Reseed interval (when to advance the simulation for new entropy)
//! - Safe step limit (from Lyapunov estimation)
//! - Bytes encrypted per key (to prevent overuse)

use sha3::{Digest, Sha3_512};

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
/// Each key is 32 bytes (ChaCha20 key) + 12 bytes (ChaCha20 nonce).
#[derive(Clone, Debug)]
pub struct KeySchedule {
    /// Current seed material.
    seed: [u8; 64],
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
    /// `seed` is the initial 64-byte seed from SHA3-512 extraction.
    /// `total_steps` is the total number of simulation steps.
    /// `reseed_interval` is the number of steps between reseeds.
    /// `safe_steps` is the maximum safe steps from Lyapunov estimation.
    pub fn new(
        seed: [u8; 64],
        total_steps: u64,
        reseed_interval: u64,
        safe_steps: u64,
    ) -> Self {
        // Each reseed produces one key, and each key can encrypt ~256 GiB
        // (ChaCha20 limit). We limit to safe_steps / reseed_interval keys.
        let max_keys = if reseed_interval > 0 {
            (safe_steps / reseed_interval).max(1)
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

    /// Get the next key and nonce.
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

        // Derive key from current seed
        let mut hasher = Sha3_512::new();
        hasher.update(b"kelvin-key-derivation-v1");
        hasher.update(&self.seed);
        hasher.update(&self.keys_generated.to_le_bytes());
        let hash = hasher.finalize();

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash[..32]);

        let mut nonce = [0u8; 12];
        nonce.copy_from_slice(&hash[32..44]);

        // Advance step and reseed if needed
        self.step += self.reseed_interval;
        self.keys_generated += 1;

        // Reseed: derive new seed from current seed
        let mut reseed_hasher = Sha3_512::new();
        reseed_hasher.update(b"kelvin-reseed-v1");
        reseed_hasher.update(&self.seed);
        reseed_hasher.update(&self.step.to_le_bytes());
        let new_hash = reseed_hasher.finalize();
        self.seed.copy_from_slice(&new_hash);

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
    pub fn reset(&mut self, seed: [u8; 64]) {
        self.seed = seed;
        self.step = 0;
        self.keys_generated = 0;
        self.state = ScheduleState::Active;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_seed() -> [u8; 64] {
        let mut seed = [0u8; 64];
        for i in 0..64 {
            seed[i] = i as u8;
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
    fn test_key_nonce_different() {
        let mut schedule = KeySchedule::new(test_seed(), 10000, 1000, 5000);
        let (key, nonce) = schedule.next_key().unwrap();
        // Key and nonce should be different (different parts of hash)
        assert_ne!(&key[..12], &nonce[..]);
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
}
