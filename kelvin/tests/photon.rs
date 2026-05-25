//! Photon cipher integration tests.
//!
//! Tests the KelvinPhoton encrypt/decrypt API: round-trip, determinism,
//! large data, bytes_processed, reseed_count, exhaustion, remaining_reseeds,
//! and avalanche effect.

use kelvin::{KelvinPhoton, PHOTON_BASE_SEED_SIZE};

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

#[test]
fn test_round_trip_small() {
    let mut photon = KelvinPhoton::new(test_seed(), 1000);
    let mut data = b"Hello, Kelvin V3 Photon!".to_vec();
    let original = data.clone();

    photon.encrypt(&mut data).unwrap();
    assert_ne!(data, original, "encrypted data should differ from plaintext");

    // Decrypt with new instance (same seed = same keystream)
    let mut photon2 = KelvinPhoton::new(test_seed(), 1000);
    photon2.decrypt(&mut data).unwrap();
    assert_eq!(data, original, "round-trip should restore original");
}

#[test]
fn test_determinism() {
    let mut p1 = KelvinPhoton::new(test_seed(), 1000);
    let mut p2 = KelvinPhoton::new(test_seed(), 1000);

    let mut data1 = b"Determinism test".to_vec();
    let mut data2 = data1.clone();

    p1.encrypt(&mut data1).unwrap();
    p2.encrypt(&mut data2).unwrap();
    assert_eq!(data1, data2, "two instances should produce identical ciphertext");
}

#[test]
fn test_empty_data() {
    let mut photon = KelvinPhoton::new(test_seed(), 1000);
    let mut empty: Vec<u8> = vec![];
    photon.encrypt(&mut empty).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_large_data() {
    let mut photon = KelvinPhoton::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 100_000]; // 100KB
    let original = data.clone();

    photon.encrypt(&mut data).unwrap();
    assert_ne!(data, original);

    let mut photon2 = KelvinPhoton::new(test_seed(), 1000);
    photon2.decrypt(&mut data).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_bytes_processed() {
    let mut photon = KelvinPhoton::new(test_seed(), 1000);
    assert_eq!(photon.bytes_processed(), 0);

    let mut data = vec![0u8; 100];
    photon.encrypt(&mut data).unwrap();
    assert_eq!(photon.bytes_processed(), 100);

    let mut data2 = vec![0u8; 50];
    photon.encrypt(&mut data2).unwrap();
    assert_eq!(photon.bytes_processed(), 150);
}

#[test]
fn test_reseed_count_increments() {
    let mut photon = KelvinPhoton::new(test_seed(), 1000);
    assert_eq!(photon.reseed_count(), 0);

    // First call initializes the reader (reseed_count becomes 1)
    let mut data = vec![0u8; 1];
    photon.encrypt(&mut data).unwrap();
    assert_eq!(photon.reseed_count(), 1);

    // Second call uses the persistent reader (no reseed)
    photon.encrypt(&mut data).unwrap();
    assert_eq!(photon.reseed_count(), 1);
}

#[test]
fn test_exhaustion() {
    let mut photon = KelvinPhoton::new(test_seed(), 3);
    // Encrypt enough to trigger 3 reseeds (each reseed lasts 64 MiB)
    // 3 reseeds = 192 MiB of keystream
    let mut data = vec![0u8; 64 * 1024 * 1024 + 1]; // 64 MiB + 1 byte

    // First 64 MiB + 1 byte: triggers reseed at 64 MiB boundary
    assert!(photon.encrypt(&mut data).is_ok());
    assert_eq!(photon.reseed_count(), 2); // initial + 1 reseed

    // Second 64 MiB + 1 byte: triggers another reseed
    assert!(photon.encrypt(&mut data).is_ok());
    assert_eq!(photon.reseed_count(), 3); // exhausted

    // Third call should fail (max_reseeds = 3, so reseed_count 3 = exhausted)
    assert!(photon.encrypt(&mut data).is_err());
}

#[test]
fn test_remaining_reseeds() {
    let mut photon = KelvinPhoton::new(test_seed(), 10);
    assert_eq!(photon.remaining_reseeds(), 10);

    // First call initializes the reader (consumes 1 reseed)
    let mut data = vec![0u8; 1];
    photon.encrypt(&mut data).unwrap();
    assert_eq!(photon.remaining_reseeds(), 9);

    // Second call uses persistent reader (no reseed consumed)
    photon.encrypt(&mut data).unwrap();
    assert_eq!(photon.remaining_reseeds(), 9);
}

#[test]
fn test_avalanche() {
    // 1-bit change in seed should produce completely different keystream
    let mut seed2 = test_seed();
    seed2[0] ^= 0x01;

    let mut p1 = KelvinPhoton::new(test_seed(), 1000);
    let mut p2 = KelvinPhoton::new(seed2, 1000);

    let mut data1 = b"Avalanche test data".to_vec();
    let mut data2 = data1.clone();

    p1.encrypt(&mut data1).unwrap();
    p2.encrypt(&mut data2).unwrap();

    let diff_bits: u32 = data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
}
