//! Benchmark: Key schedule derivation — varying seed material.
//!
//! Tests the `KeySchedule::next_key` method which uses HKDF-SHA512 for
//! key derivation and BLAKE3 for reseeding. Both HKDF and BLAKE3 should
//! be constant-time, but this benchmark verifies that the composition
//! with step counting and exhaustion checks doesn't introduce timing
//! variations.
//!
//! ## Classes
//!
//! - **Left (small seed values):** Seed bytes are in range 0x00–0x7F
//! - **Right (large seed values):** Seed bytes are in range 0x80–0xFF
//!
//! Both classes generate the first key from a fresh schedule, isolating
//! the seed material variation as the only difference between classes.

use dudect_bencher::{rand::RngExt, BenchRng, Class, CtRunner};
use kelvin_kdf::KeySchedule;

/// Number of test vectors per benchmark run.
const NUM_SAMPLES: usize = 50_000;

/// Benchmark key_schedule: small vs. large seed values.
pub fn bench_key_schedule(runner: &mut CtRunner, rng: &mut BenchRng) {
    let mut classes = Vec::with_capacity(NUM_SAMPLES);
    let mut seeds = Vec::with_capacity(NUM_SAMPLES);

    for _ in 0..NUM_SAMPLES {
        let mut seed = [0u8; 2048];

        if rng.random::<bool>() {
            // Left class: small seed values (0x00–0x7F)
            for byte in seed.iter_mut() {
                *byte = rng.random_range(0u8..=0x7F);
            }
            classes.push(Class::Left);
        } else {
            // Right class: large seed values (0x80–0xFF)
            for byte in seed.iter_mut() {
                *byte = rng.random_range(0x80u8..=0xFF);
            }
            classes.push(Class::Right);
        }
        seeds.push(seed);
    }

    for (seed, class) in seeds.into_iter().zip(classes) {
        runner.run_one(class, || {
            let mut schedule = KeySchedule::new(seed, 100_000, 10_000, 50_000);
            let _result = schedule.next_key();
        });
    }
}
