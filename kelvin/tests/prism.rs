//! Prism cipher integration tests.
//!
//! Tests the KelvinPrism OTP key generator API: round-trip, determinism,
//! generate_otp_key, split_key, recrypt, domain separation, large data,
//! bytes_processed, reseed_count, exhaustion, and remaining_reseeds.

use kelvin::{KelvinPhoton, KelvinPrism, PHOTON_BASE_SEED_SIZE};

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

#[test]
fn test_round_trip_small() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);
    let mut data = b"Hello, Kelvin Prism!".to_vec();
    let original = data.clone();

    prism.encrypt(&mut data).unwrap();
    assert_ne!(data, original, "encrypted data should differ from plaintext");

    // Decrypt with new instance (same seed = same keystream)
    let mut prism2 = KelvinPrism::new(test_seed(), 1000);
    prism2.decrypt(&mut data).unwrap();
    assert_eq!(data, original, "round-trip should restore original");
}

#[test]
fn test_determinism() {
    let mut p1 = KelvinPrism::new(test_seed(), 1000);
    let mut p2 = KelvinPrism::new(test_seed(), 1000);

    let mut data1 = b"Determinism test".to_vec();
    let mut data2 = data1.clone();

    p1.encrypt(&mut data1).unwrap();
    p2.encrypt(&mut data2).unwrap();
    assert_eq!(data1, data2, "two instances should produce identical ciphertext");
}

#[test]
fn test_empty_data() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);
    let mut empty: Vec<u8> = vec![];
    prism.encrypt(&mut empty).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_generate_otp_key() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);

    // Generate a 32-byte OTP key
    let key = prism.generate_otp_key(32).unwrap();
    assert_eq!(key.len(), 32);

    // Key should not be all zeros
    assert!(key.iter().any(|&b| b != 0), "OTP key should not be all zeros");

    // Generate a 256-byte OTP key
    let key256 = prism.generate_otp_key(256).unwrap();
    assert_eq!(key256.len(), 256);

    // Different lengths should produce different keys
    let key2 = prism.generate_otp_key(32).unwrap();
    assert_ne!(key, key2, "subsequent keys should differ");
}

#[test]
fn test_split_key() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);

    // Split a 256-byte key into (A, B) where A ⊕ B = K
    let (a, b) = prism.split_key(256).unwrap();
    assert_eq!(a.len(), 256);
    assert_eq!(b.len(), 256);

    // Verify A ⊕ B = K (the original key)
    let xor_result: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
    assert!(xor_result.iter().any(|&b| b != 0), "A ⊕ B should not be all zeros");

    // Verify that A and B are different
    assert_ne!(a, b, "split pads should differ");
}

#[test]
fn test_recrypt_static() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);

    // Generate an OTP key
    let otp_key = prism.generate_otp_key(32).unwrap();

    // Encrypt data with static recryption
    let mut data = b"Static recryption test".to_vec();
    let original = data.clone();

    KelvinPrism::recrypt(&mut data, &otp_key);
    assert_ne!(data, original, "recrypted data should differ from plaintext");

    // Decrypt with the same key
    KelvinPrism::recrypt(&mut data, &otp_key);
    assert_eq!(data, original, "double recryption should restore original");
}

#[test]
fn test_domain_separation() {
    // Prism and Photon with the same seed should produce different keystream
    let seed = test_seed();

    let mut prism = KelvinPrism::new(seed, 1000);
    let mut photon = KelvinPhoton::new(seed, 1000);

    let mut data_prism = b"Domain separation test".to_vec();
    let mut data_photon = data_prism.clone();

    prism.encrypt(&mut data_prism).unwrap();
    photon.encrypt(&mut data_photon).unwrap();

    assert_ne!(data_prism, data_photon, "Prism and Photon keystream should be domain-separated");
}

#[test]
fn test_large_data() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 100_000]; // 100KB
    let original = data.clone();

    prism.encrypt(&mut data).unwrap();
    assert_ne!(data, original);

    let mut prism2 = KelvinPrism::new(test_seed(), 1000);
    prism2.decrypt(&mut data).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_bytes_processed() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);
    assert_eq!(prism.bytes_processed(), 0);

    let mut data = vec![0u8; 100];
    prism.encrypt(&mut data).unwrap();
    assert_eq!(prism.bytes_processed(), 100);

    let mut data2 = vec![0u8; 50];
    prism.encrypt(&mut data2).unwrap();
    assert_eq!(prism.bytes_processed(), 150);
}

#[test]
fn test_reseed_count_increments() {
    let mut prism = KelvinPrism::new(test_seed(), 1000);
    assert_eq!(prism.reseed_count(), 0);

    // First call initializes the reader (reseed_count becomes 1)
    let mut data = vec![0u8; 1];
    prism.encrypt(&mut data).unwrap();
    assert_eq!(prism.reseed_count(), 1);

    // Second call uses the persistent reader (no reseed)
    prism.encrypt(&mut data).unwrap();
    assert_eq!(prism.reseed_count(), 1);
}

#[test]
fn test_exhaustion() {
    let mut prism = KelvinPrism::new(test_seed(), 3);
    // Encrypt enough to trigger 3 reseeds (each reseed lasts 64 MiB)
    // 3 reseeds = 192 MiB of keystream
    let mut data = vec![0u8; 64 * 1024 * 1024 + 1]; // 64 MiB + 1 byte

    // First 64 MiB + 1 byte: triggers reseed at 64 MiB boundary
    assert!(prism.encrypt(&mut data).is_ok());
    assert_eq!(prism.reseed_count(), 2); // initial + 1 reseed

    // Second 64 MiB + 1 byte: triggers another reseed
    assert!(prism.encrypt(&mut data).is_ok());
    assert_eq!(prism.reseed_count(), 3); // exhausted

    // Third call should fail (max_reseeds = 3, so reseed_count 3 = exhausted)
    assert!(prism.encrypt(&mut data).is_err());
}

#[test]
fn test_remaining_reseeds() {
    let mut prism = KelvinPrism::new(test_seed(), 10);
    assert_eq!(prism.remaining_reseeds(), 10);

    // First call initializes the reader (consumes 1 reseed)
    let mut data = vec![0u8; 1];
    prism.encrypt(&mut data).unwrap();
    assert_eq!(prism.remaining_reseeds(), 9);

    // Second call uses persistent reader (no reseed consumed)
    prism.encrypt(&mut data).unwrap();
    assert_eq!(prism.remaining_reseeds(), 9);
}

#[test]
fn test_avalanche() {
    // 1-bit change in seed should produce completely different keystream
    let mut seed2 = test_seed();
    seed2[0] ^= 0x01;

    let mut p1 = KelvinPrism::new(test_seed(), 1000);
    let mut p2 = KelvinPrism::new(seed2, 1000);

    let mut data1 = b"Avalanche test data".to_vec();
    let mut data2 = data1.clone();

    p1.encrypt(&mut data1).unwrap();
    p2.encrypt(&mut data2).unwrap();

    let diff_bits: u32 = data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
}

#[test]
fn test_generate_otp_key_determinism() {
    // Same seed + same reseed_count should produce same OTP key
    let mut p1 = KelvinPrism::new(test_seed(), 1000);
    let mut p2 = KelvinPrism::new(test_seed(), 1000);

    let key1 = p1.generate_otp_key(64).unwrap();
    let key2 = p2.generate_otp_key(64).unwrap();

    assert_eq!(key1, key2, "OTP keys should be deterministic from same seed");
}

#[test]
fn test_split_key_xor_property() {
    // Verify that split_key produces (A, B) where A ⊕ B = K
    // and that K can be recovered by XORing A and B
    let mut prism = KelvinPrism::new(test_seed(), 1000);

    let (a, b) = prism.split_key(128).unwrap();

    // A ⊕ B should equal the original key K
    let k_recovered: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();

    // Generate another key from a fresh instance to verify K is deterministic
    let mut prism2 = KelvinPrism::new(test_seed(), 1000);
    // Skip the first generate_otp_key call (split_key consumed one)
    let _first = prism2.generate_otp_key(128).unwrap();
    let k_expected = prism2.generate_otp_key(128).unwrap();

    assert_eq!(k_recovered, k_expected, "A ⊕ B should equal the original key K");
}
