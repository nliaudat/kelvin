//! Quantum cipher integration tests.
//!
//! Tests the KelvinQuantum encrypt/decrypt API: round-trip, determinism,
//! large data, bytes_processed, reseed_count, exhaustion, remaining_reseeds,
//! and avalanche effect.

use kelvin::{KelvinQuantum, QUANTUM_BASE_SEED_SIZE, QUANTUM_DEFAULT_CACHE_SIZE};

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

    let diff_bits: u32 = data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
}
