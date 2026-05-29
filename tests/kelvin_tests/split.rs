//! Split cipher integration tests.
//!
//! Tests the KelvinSplit XOR key-splitter API: round-trip, determinism,
//! split_key, generate_master_key, domain separation from Prism/Photon/Flare,
//! large data, bytes_processed, reseed_count, exhaustion, remaining_reseeds,
//! avalanche effect, and split_key XOR property.

use kelvin::{KelvinFlare, KelvinPhoton, KelvinPrism, KelvinSplit, PHOTON_BASE_SEED_SIZE};

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

#[test]
fn test_round_trip_small() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    let mut data = b"Hello, Kelvin Split!".to_vec();
    let original = data.clone();

    split.encrypt(&mut data).unwrap();
    assert_ne!(data, original, "encrypted data should differ from plaintext");

    // Decrypt with new instance (same seed = same keystream)
    let mut split2 = KelvinSplit::new(test_seed(), 1000);
    split2.decrypt(&mut data).unwrap();
    assert_eq!(data, original, "round-trip should restore original");
}

#[test]
fn test_determinism() {
    let mut s1 = KelvinSplit::new(test_seed(), 1000);
    let mut s2 = KelvinSplit::new(test_seed(), 1000);

    let mut data1 = b"Determinism test".to_vec();
    let mut data2 = data1.clone();

    s1.encrypt(&mut data1).unwrap();
    s2.encrypt(&mut data2).unwrap();
    assert_eq!(data1, data2, "two instances should produce identical ciphertext");
}

#[test]
fn test_empty_data() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    let mut empty: Vec<u8> = vec![];
    split.encrypt(&mut empty).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_generate_master_key() {
    let mut split = KelvinSplit::new(test_seed(), 1000);

    // Generate a 32-byte master key
    let key = split.generate_master_key(32).unwrap();
    assert_eq!(key.len(), 32);

    // Key should not be all zeros
    assert!(key.iter().any(|&b| b != 0), "master key should not be all zeros");

    // Generate a 256-byte master key
    let key256 = split.generate_master_key(256).unwrap();
    assert_eq!(key256.len(), 256);

    // Different lengths should produce different keys
    let key2 = split.generate_master_key(32).unwrap();
    assert_ne!(key, key2, "subsequent keys should differ");
}

#[test]
fn test_split_key() {
    let mut split = KelvinSplit::new(test_seed(), 1000);

    // Split a 256-byte key into (A, B) where A ⊕ B = K
    let (a, b) = split.split_key(256).unwrap();
    assert_eq!(a.len(), 256);
    assert_eq!(b.len(), 256);

    // Verify A ⊕ B = K (the original key)
    let xor_result: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();
    assert!(xor_result.iter().any(|&b| b != 0), "A ⊕ B should not be all zeros");

    // Verify that A and B are different
    assert_ne!(a, b, "split pads should differ");
}

#[test]
fn test_split_key_xor_property() {
    // Verify that split_key produces (A, B) where A ⊕ B = K
    // and that K can be recovered by XORing A and B
    let mut split = KelvinSplit::new(test_seed(), 1000);

    let (a, b) = split.split_key(128).unwrap();

    // A ⊕ B should equal the original key K
    let k_recovered: Vec<u8> = a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect();

    // Generate another key from a fresh instance to verify K is deterministic.
    // split_key internally generates K (len bytes via generate_master_key),
    // then A (len bytes via generate_keystream_into). So K is the first
    // len bytes of keystream from a fresh instance.
    let mut split2 = KelvinSplit::new(test_seed(), 1000);
    let k_expected = split2.generate_master_key(128).unwrap();

    assert_eq!(k_recovered, *k_expected, "A ⊕ B should equal the original key K");
}

#[test]
fn test_large_data() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 100_000]; // 100KB
    let original = data.clone();

    split.encrypt(&mut data).unwrap();
    assert_ne!(data, original);

    let mut split2 = KelvinSplit::new(test_seed(), 1000);
    split2.decrypt(&mut data).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_bytes_processed() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    assert_eq!(split.bytes_processed(), 0);

    let mut data = vec![0u8; 100];
    split.encrypt(&mut data).unwrap();
    assert_eq!(split.bytes_processed(), 100);

    let mut data2 = vec![0u8; 50];
    split.encrypt(&mut data2).unwrap();
    assert_eq!(split.bytes_processed(), 150);
}

#[test]
fn test_reseed_count_increments() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    assert_eq!(split.reseed_count(), 0);

    // First call initializes the reader (reseed_count becomes 1)
    let mut data = vec![0u8; 1];
    split.encrypt(&mut data).unwrap();
    assert_eq!(split.reseed_count(), 1);

    // Second call uses the persistent reader (no reseed)
    split.encrypt(&mut data).unwrap();
    assert_eq!(split.reseed_count(), 1);
}

#[test]
fn test_exhaustion() {
    let mut split = KelvinSplit::new(test_seed(), 3);
    // Encrypt enough to trigger 3 reseeds (each reseed lasts 64 MiB)
    // 3 reseeds = 192 MiB of keystream. Use a small reusable buffer (1 MiB)
    // in a loop to trigger reseeds without large allocations.
    let mut buf = vec![0u8; 1024 * 1024]; // 1 MiB buffer

    // First 64 MiB + 1 byte: triggers reseed at 64 MiB boundary
    for _ in 0..64 {
        split.generate_keystream_into(&mut buf).unwrap();
    }
    split.generate_keystream_into(&mut buf[..1]).unwrap();
    assert_eq!(split.reseed_count(), 2); // initial + 1 reseed

    // Second 64 MiB + 1 byte: triggers another reseed
    for _ in 0..64 {
        split.generate_keystream_into(&mut buf).unwrap();
    }
    split.generate_keystream_into(&mut buf[..1]).unwrap();
    assert_eq!(split.reseed_count(), 3); // exhausted

    // Third interval: process 64 MiB to trigger the next reseed check,
    // which should fail (max_reseeds = 3, so reseed_count 3 = exhausted)
    for _ in 0..64 {
        split.generate_keystream_into(&mut buf).unwrap();
    }
    assert!(split.generate_keystream_into(&mut buf[..1]).is_err());
}

#[test]
fn test_remaining_reseeds() {
    let mut split = KelvinSplit::new(test_seed(), 10);
    assert_eq!(split.remaining_reseeds(), 10);

    // First call initializes the reader (consumes 1 reseed)
    let mut data = vec![0u8; 1];
    split.encrypt(&mut data).unwrap();
    assert_eq!(split.remaining_reseeds(), 9);

    // Second call uses persistent reader (no reseed consumed)
    split.encrypt(&mut data).unwrap();
    assert_eq!(split.remaining_reseeds(), 9);
}

#[test]
fn test_avalanche() {
    // 1-bit change in seed should produce completely different keystream
    let mut seed2 = test_seed();
    seed2[0] ^= 0x01;

    let mut s1 = KelvinSplit::new(test_seed(), 1000);
    let mut s2 = KelvinSplit::new(seed2, 1000);

    let mut data1 = b"Avalanche test data".to_vec();
    let mut data2 = data1.clone();

    s1.encrypt(&mut data1).unwrap();
    s2.encrypt(&mut data2).unwrap();

    let diff_bits: u32 = data1.iter().zip(data2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    assert!(diff_bits > 50, "Too few differing bits: {}", diff_bits);
}

#[test]
fn test_generate_master_key_determinism() {
    // Same seed + same reseed_count should produce same master key
    let mut s1 = KelvinSplit::new(test_seed(), 1000);
    let mut s2 = KelvinSplit::new(test_seed(), 1000);

    let key1 = s1.generate_master_key(64).unwrap();
    let key2 = s2.generate_master_key(64).unwrap();

    assert_eq!(key1, key2, "master keys should be deterministic from same seed");
}

#[test]
fn test_domain_separation_from_prism() {
    let seed = test_seed();
    let mut split = KelvinSplit::new(seed, 1000);
    let mut prism = KelvinPrism::new(seed, 1000);

    let split_key = split.generate_master_key(64).unwrap();
    let prism_key = prism.generate_otp_key(64).unwrap();

    assert_ne!(&*split_key, &*prism_key, "Split and Prism keystream should be domain-separated");
}

#[test]
fn test_domain_separation_from_photon() {
    let seed = test_seed();
    let mut split = KelvinSplit::new(seed, 1000);
    let mut photon = KelvinPhoton::new(seed, 1000);

    let split_key = split.generate_master_key(64).unwrap();
    let mut photon_buf = vec![0u8; 64];
    photon.encrypt(&mut photon_buf).unwrap();

    assert_ne!(
        &*split_key,
        &photon_buf[..],
        "Split and Photon keystream should be domain-separated"
    );
}

#[test]
fn test_domain_separation_from_flare() {
    let seed = test_seed();
    let mut split = KelvinSplit::new(seed, 1000);
    let mut flare = KelvinFlare::new(seed, 1000);

    let split_key = split.generate_master_key(64).unwrap();
    let flare_key = flare.generate_secret_key(64).unwrap();

    assert_ne!(&*split_key, &*flare_key, "Split and Flare keystream should be domain-separated");
}

#[test]
fn test_split_key_zero_length() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    let (a, b) = split.split_key(0).unwrap();
    assert!(a.is_empty());
    assert!(b.is_empty());
}

#[test]
fn test_generate_master_key_zero_length() {
    let mut split = KelvinSplit::new(test_seed(), 1000);
    let key = split.generate_master_key(0).unwrap();
    assert!(key.is_empty());
}
