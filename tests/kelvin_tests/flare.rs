//! Flare cipher integration tests.
//!
//! Tests the KelvinFlare chaotic FHE secret key generator API:
//! round-trip, determinism, generate_secret_key, generate_fhe_key,
//! FlareScheme isolation, domain separation from Split/Prism/Photon,
//! large data, bytes_processed, reseed_count, exhaustion,
//! remaining_reseeds, avalanche effect, and edge cases.

use kelvin::{
    FlareScheme, KelvinFlare, KelvinPhoton, KelvinPrism, KelvinSplit, PHOTON_BASE_SEED_SIZE,
};

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

#[test]
fn test_generate_secret_key() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);

    // Generate a 32-byte secret key
    let key = flare.generate_secret_key(32).unwrap();
    assert_eq!(key.len(), 32);

    // Key should not be all zeros
    assert!(key.iter().any(|&b| b != 0), "secret key should not be all zeros");

    // Generate a 256-byte secret key
    let key256 = flare.generate_secret_key(256).unwrap();
    assert_eq!(key256.len(), 256);

    // Subsequent keys should differ
    let key2 = flare.generate_secret_key(32).unwrap();
    assert_ne!(key, key2, "subsequent keys should differ");
}

#[test]
fn test_generate_secret_key_determinism() {
    let mut f1 = KelvinFlare::new(test_seed(), 1000);
    let mut f2 = KelvinFlare::new(test_seed(), 1000);

    let key1 = f1.generate_secret_key(64).unwrap();
    let key2 = f2.generate_secret_key(64).unwrap();

    assert_eq!(key1, key2, "secret keys should be deterministic from same seed");
}

#[test]
fn test_generate_fhe_key_bfv() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let fhe_key = flare.generate_fhe_key(FlareScheme::Bfv, 64).unwrap();

    assert_eq!(fhe_key.scheme(), FlareScheme::Bfv);
    assert_eq!(fhe_key.len(), 64);
    assert!(!fhe_key.is_empty());
    assert!(fhe_key.key().iter().any(|&b| b != 0), "BFV key should not be all zeros");
}

#[test]
fn test_generate_fhe_key_ckks() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let fhe_key = flare.generate_fhe_key(FlareScheme::Ckks, 64).unwrap();

    assert_eq!(fhe_key.scheme(), FlareScheme::Ckks);
    assert_eq!(fhe_key.len(), 64);
    assert!(!fhe_key.is_empty());
}

#[test]
fn test_generate_fhe_key_tfhe() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let fhe_key = flare.generate_fhe_key(FlareScheme::Tfhe, 64).unwrap();

    assert_eq!(fhe_key.scheme(), FlareScheme::Tfhe);
    assert_eq!(fhe_key.len(), 64);
    assert!(!fhe_key.is_empty());
}

#[test]
fn test_fhe_key_scheme_isolation() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);

    let bfv_key = flare.generate_fhe_key(FlareScheme::Bfv, 64).unwrap();
    let ckks_key = flare.generate_fhe_key(FlareScheme::Ckks, 64).unwrap();
    let tfhe_key = flare.generate_fhe_key(FlareScheme::Tfhe, 64).unwrap();

    // Keys for different schemes should be different
    assert_ne!(bfv_key.key(), ckks_key.key(), "BFV and CKKS keys should differ");
    assert_ne!(bfv_key.key(), tfhe_key.key(), "BFV and TFHE keys should differ");
    assert_ne!(ckks_key.key(), tfhe_key.key(), "CKKS and TFHE keys should differ");
}

#[test]
fn test_fhe_key_scheme_isolation_deterministic() {
    // Same scheme + same seed + same position = same key
    let mut f1 = KelvinFlare::new(test_seed(), 1000);
    let mut f2 = KelvinFlare::new(test_seed(), 1000);

    let bfv1 = f1.generate_fhe_key(FlareScheme::Bfv, 32).unwrap();
    let bfv2 = f2.generate_fhe_key(FlareScheme::Bfv, 32).unwrap();

    assert_eq!(bfv1.key(), bfv2.key(), "same scheme should produce same key from same seed");
}

#[test]
fn test_generate_fhe_key_zero_length() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let fhe_key = flare.generate_fhe_key(FlareScheme::Bfv, 0).unwrap();

    assert!(fhe_key.is_empty());
    assert_eq!(fhe_key.len(), 0);
}

#[test]
fn test_generate_secret_key_zero_length() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let key = flare.generate_secret_key(0).unwrap();
    assert!(key.is_empty());
}

#[test]
fn test_large_key() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);

    // Generate a 1 MB key
    let key = flare.generate_secret_key(1024 * 1024).unwrap();
    assert_eq!(key.len(), 1024 * 1024);
    assert!(key.iter().any(|&b| b != 0), "large key should not be all zeros");
}

#[test]
fn test_bytes_processed() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    assert_eq!(flare.bytes_processed(), 0);

    flare.generate_secret_key(100).unwrap();
    assert_eq!(flare.bytes_processed(), 100);

    flare.generate_secret_key(200).unwrap();
    assert_eq!(flare.bytes_processed(), 300);
}

#[test]
fn test_reseed_count_increments() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    assert_eq!(flare.reseed_count(), 0);

    // First call initializes the reader (reseed_count becomes 1)
    flare.generate_secret_key(1).unwrap();
    assert_eq!(flare.reseed_count(), 1);

    // Second call uses the persistent reader (no reseed)
    flare.generate_secret_key(1).unwrap();
    assert_eq!(flare.reseed_count(), 1);
}

#[test]
fn test_exhaustion() {
    let mut flare = KelvinFlare::new(test_seed(), 3);

    // Each reseed lasts 64 MiB. With max_reseeds=3, we can generate
    // 3 * 64 MiB before exhaustion. Use a small reusable buffer (1 MiB)
    // in a loop to trigger reseeds without large allocations.
    let mut buf = vec![0u8; 1024 * 1024]; // 1 MiB buffer

    // Call 1: Generates 64 MiB to trigger first reseed
    for _ in 0..64 {
        flare.generate_keystream_into(&mut buf).unwrap();
    }
    assert_eq!(flare.reseed_count(), 1);

    // Call 2: Generates another 64 MiB to trigger second reseed
    for _ in 0..64 {
        flare.generate_keystream_into(&mut buf).unwrap();
    }
    assert_eq!(flare.reseed_count(), 2);

    // Call 3: Generates another 64 MiB to trigger third reseed
    for _ in 0..64 {
        flare.generate_keystream_into(&mut buf).unwrap();
    }
    assert_eq!(flare.reseed_count(), 3);

    // Call 4: reseed_count (3) >= max_reseeds (3) -> SeedExhausted
    assert!(flare.generate_keystream_into(&mut buf[..1]).is_err());
}

#[test]
fn test_remaining_reseeds() {
    let mut flare = KelvinFlare::new(test_seed(), 10);
    assert_eq!(flare.remaining_reseeds(), 10);

    // First call initializes the reader (consumes 1 reseed)
    flare.generate_secret_key(1).unwrap();
    assert_eq!(flare.remaining_reseeds(), 9);

    // Second call uses persistent reader (no reseed consumed)
    flare.generate_secret_key(1).unwrap();
    assert_eq!(flare.remaining_reseeds(), 9);
}

#[test]
fn test_avalanche() {
    // 1-bit change in seed should produce completely different keystream
    let mut seed2 = test_seed();
    seed2[0] ^= 0x01;

    let mut f1 = KelvinFlare::new(test_seed(), 1000);
    let mut f2 = KelvinFlare::new(seed2, 1000);

    let key1 = f1.generate_secret_key(64).unwrap();
    let key2 = f2.generate_secret_key(64).unwrap();

    let diff_bits: u32 = key1.iter().zip(key2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();
    assert!(diff_bits > 200, "Too few differing bits: {}", diff_bits);
}

#[test]
fn test_domain_separation_from_split() {
    let seed = test_seed();
    let mut flare = KelvinFlare::new(seed, 1000);
    let mut split = KelvinSplit::new(seed, 1000);

    let flare_key = flare.generate_secret_key(64).unwrap();
    let split_key = split.generate_master_key(64).unwrap();

    assert_ne!(&*flare_key, &*split_key, "Flare and Split keystream should be domain-separated");
}

#[test]
fn test_domain_separation_from_prism() {
    let seed = test_seed();
    let mut flare = KelvinFlare::new(seed, 1000);
    let mut prism = KelvinPrism::new(seed, 1000);

    let flare_key = flare.generate_secret_key(64).unwrap();
    let prism_key = prism.generate_otp_key(64).unwrap();

    assert_ne!(&*flare_key, &*prism_key, "Flare and Prism keystream should be domain-separated");
}

#[test]
fn test_domain_separation_from_photon() {
    let seed = test_seed();
    let mut flare = KelvinFlare::new(seed, 1000);
    let mut photon = KelvinPhoton::new(seed, 1000);

    let flare_key = flare.generate_secret_key(64).unwrap();
    let mut photon_buf = vec![0u8; 64];
    photon.encrypt(&mut photon_buf).unwrap();

    assert_ne!(
        &*flare_key,
        &photon_buf[..],
        "Flare and Photon keystream should be domain-separated"
    );
}

#[test]
fn test_flare_key_metadata() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let fhe_key = flare.generate_fhe_key(FlareScheme::Bfv, 128).unwrap();

    assert_eq!(fhe_key.scheme(), FlareScheme::Bfv);
    assert_eq!(fhe_key.len(), 128);
    assert!(!fhe_key.is_empty());
    assert_eq!(fhe_key.key().len(), 128);
}

#[test]
fn test_flare_key_clone() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let key1 = flare.generate_fhe_key(FlareScheme::Ckks, 64).unwrap();
    let key2 = key1.clone();

    assert_eq!(key1.key(), key2.key());
    assert_eq!(key1.scheme(), key2.scheme());
    assert_eq!(key1.len(), key2.len());
}

#[test]
fn test_flare_key_debug() {
    let mut flare = KelvinFlare::new(test_seed(), 1000);
    let key = flare.generate_fhe_key(FlareScheme::Tfhe, 32).unwrap();
    let debug_str = format!("{:?}", key);
    assert!(!debug_str.is_empty());
}

#[test]
fn test_debug_redacts_seed() {
    let flare = KelvinFlare::new(test_seed(), 1000);
    let debug_str = format!("{:?}", flare);
    assert!(!debug_str.contains("0u8"));
    assert!(debug_str.contains("[redacted]"));
}
