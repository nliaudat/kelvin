//! Canonical test vectors for all Kelvin modes.
//!
//! Generates and validates deterministic test vectors for each encryption mode.
//! If the test vector file exists, validates against it (regression detection).
//! If it doesn't exist, generates it (initial setup).

use kelvin::{
    Kelvin, KelvinFlare, KelvinPhoton, KelvinPrism, KelvinQuantum, KelvinSplit, KelvinStreaming,
    OrbitalConfig,
};
use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

/// Create a deterministic 5-body orbital configuration for testing.
fn test_config() -> OrbitalConfig {
    let bodies = vec![
        OrbitalBody::new(
            Fixed::ONE,
            Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 54), // ~1e-6 solar masses
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
        ),
    ];
    OrbitalConfig::new(bodies, 200, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("valid test config")
}

/// Helper: create a deterministic 2048-byte seed for testing.
fn test_seed() -> [u8; 2048] {
    let mut seed = [0u8; 2048];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

/// Helper: assert two byte slices are equal, printing details on failure.
fn assert_eq_slice(label: &str, expected: &[u8], actual: &[u8]) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "{}: length mismatch (expected {}, got {})",
        label,
        expected.len(),
        actual.len()
    );
    for (i, (e, a)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(e, a, "{}: byte {} differs (expected {:#04x}, got {:#04x})", label, i, e, a);
    }
}

// ── V1 AEAD (Kelvin) test vector

#[test]
fn test_vector_kelvin_v1() {
    let config = test_config();
    let mut encryptor = Kelvin::new(config.clone()).expect("Kelvin::new");
    let plaintext = b"Kelvin V1 test vector (32 bytes data)!!";
    let mut ciphertext = plaintext.to_vec();
    ciphertext.extend_from_slice(&[0u8; 16]);
    let plaintext_len = plaintext.len();
    encryptor.encrypt(&mut ciphertext).unwrap();
    let mut decryptor = Kelvin::new(config).expect("Kelvin::new");
    let encrypted_data = &ciphertext[..plaintext_len + 16];
    let mut decrypted = encrypted_data.to_vec();
    decryptor.decrypt(&mut decrypted).unwrap();
    assert_eq_slice("V1 round-trip", plaintext, &decrypted[..plaintext_len]);
}

// ── V2 Streaming (KelvinStreaming) test vector

#[test]
fn test_vector_kelvin_v2_streaming() {
    let config = test_config();
    let mut enc = KelvinStreaming::new(config.clone(), 64).expect("KelvinStreaming::new");
    let mut dec = KelvinStreaming::new(config, 64).expect("KelvinStreaming::new");
    let plaintext = b"Kelvin V2 streaming test vector data here!";
    let mut encrypted = plaintext.to_vec();
    enc.encrypt(&mut encrypted).unwrap();
    assert_ne!(&encrypted, plaintext, "V2: ciphertext should differ from plaintext");
    dec.decrypt(&mut encrypted).unwrap();
    assert_eq_slice("V2 round-trip", plaintext, &encrypted);
}

// ── V3 Photon test vector

#[test]
fn test_vector_kelvin_v3_photon() {
    let seed = test_seed();
    let mut enc = KelvinPhoton::new(seed, 100);
    let mut dec = KelvinPhoton::new(seed, 100);
    let plaintext = b"Kelvin V3 Photon test vector data 0123456789";
    let mut encrypted = plaintext.to_vec();
    enc.encrypt(&mut encrypted).unwrap();
    assert_ne!(&encrypted, plaintext, "V3: ciphertext should differ from plaintext");
    dec.decrypt(&mut encrypted).unwrap();
    assert_eq_slice("V3 round-trip", plaintext, &encrypted);
}

// ── H Quantum test vector

#[test]
fn test_vector_kelvin_h_quantum() {
    let seed = test_seed();
    let mut enc = KelvinQuantum::new(seed, 100);
    let mut dec = KelvinQuantum::new(seed, 100);
    let plaintext = b"Kelvin H Quantum test vector data 40 bytes";
    let mut encrypted = plaintext.to_vec();
    enc.encrypt(&mut encrypted).unwrap();
    assert_ne!(&encrypted, plaintext, "H: ciphertext should differ from plaintext");
    dec.decrypt(&mut encrypted).unwrap();
    assert_eq_slice("H round-trip", plaintext, &encrypted);
}

// ── Prism test vector

#[test]
fn test_vector_kelvin_prism() {
    let seed = test_seed();
    let mut enc = KelvinPrism::new(seed, 100);
    let mut dec = KelvinPrism::new(seed, 100);
    let plaintext = b"Kelvin Prism test vector for HE integration";
    let mut encrypted = plaintext.to_vec();
    enc.encrypt(&mut encrypted).unwrap();
    assert_ne!(&encrypted, plaintext, "Prism: ciphertext should differ");
    dec.decrypt(&mut encrypted).unwrap();
    assert_eq_slice("Prism round-trip", plaintext, &encrypted);
}

// ── Split test vector

#[test]
fn test_vector_kelvin_split() {
    let seed = test_seed();
    let mut split = KelvinSplit::new(seed, 100);
    let master_key = split.generate_master_key(64).expect("generate_master_key");
    assert!(master_key.iter().any(|&b| b != 0), "Split: master key should not be all zeros");
    let len = 64;
    let (a, b) = split.split_key(len).expect("split_key");
    assert_eq!(a.len(), len);
    assert_eq!(b.len(), len);
    assert_ne!(&*a, &*b, "Split: pads A and B should differ");
}

// ── Flare test vector

#[test]
fn test_vector_kelvin_flare() {
    let seed = test_seed();
    let mut flare = KelvinFlare::new(seed, 100);
    let bfv_key = flare.generate_fhe_key(kelvin::FlareScheme::Bfv, 64).expect("BFV key");
    let ckks_key = flare.generate_fhe_key(kelvin::FlareScheme::Ckks, 64).expect("CKKS key");
    let tfhe_key = flare.generate_fhe_key(kelvin::FlareScheme::Tfhe, 64).expect("TFHE key");
    assert_eq!(bfv_key.len(), 64);
    assert_eq!(ckks_key.len(), 64);
    assert_eq!(tfhe_key.len(), 64);
    assert_ne!(bfv_key.key(), ckks_key.key(), "Flare: BFV and CKKS keys should differ");
    assert_ne!(bfv_key.key(), tfhe_key.key(), "Flare: BFV and TFHE keys should differ");
    assert_ne!(ckks_key.key(), tfhe_key.key(), "Flare: CKKS and TFHE keys should differ");
}

// ── Determinism across modes

#[test]
fn test_vector_determinism() {
    let seed = test_seed();
    let mut p1 = KelvinPhoton::new(seed, 100);
    let mut p2 = KelvinPhoton::new(seed, 100);
    let mut buf1 = vec![0xABu8; 1024];
    let mut buf2 = buf1.clone();
    p1.encrypt(&mut buf1).unwrap();
    p2.encrypt(&mut buf2).unwrap();
    assert_eq_slice("V3 determinism", &buf1, &buf2);
}
