//! Authenticated encryption integration tests.
//!
//! Tests KelvinPhotonAuthenticated, KelvinQuantumAuthenticated, and
//! KelvinStreamingAuthenticated: round-trip, tamper detection, tag mismatch,
//! version mismatch, and too-short data.

use kelvin::{
    Fixed, KelvinError, KelvinPhotonAuthenticated, KelvinQuantumAuthenticated,
    KelvinStreamingAuthenticated, OrbitalBody, OrbitalConfig, Vec3, AUTH_OVERHEAD,
    PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE,
};

fn test_seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    let mut seed = [0u8; PHOTON_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

fn test_quantum_seed() -> [u8; QUANTUM_BASE_SEED_SIZE] {
    let mut seed = [0u8; QUANTUM_BASE_SEED_SIZE];
    for (i, byte) in seed.iter_mut().enumerate() {
        *byte = (i % 256) as u8;
    }
    seed
}

// ─── Photon Authenticated Tests ────────────────────────────────────────

#[test]
fn test_photon_auth_round_trip() {
    let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    // Buffer needs AUTH_OVERHEAD (33) extra bytes for version byte + KMAC128 tag
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    let original = data.clone();

    auth.encrypt(&mut data).unwrap();
    // Ciphertext portion (first 64 bytes) should differ from plaintext
    assert_ne!(&data[..64], &original[..64]);

    // Decrypt with new instance (same seed = same keystream + same MAC key)
    let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    auth2.decrypt(&mut data).unwrap();
    assert_eq!(&data[..64], &original[..64]);
}

#[test]
fn test_photon_auth_tamper_detection() {
    let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    auth.encrypt(&mut data).unwrap();

    // Tamper with the ciphertext
    data[0] ^= 0x01;

    // Decryption should fail
    let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    assert!(auth2.decrypt(&mut data).is_err());
}

#[test]
fn test_photon_auth_tag_mismatch() {
    let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    auth.encrypt(&mut data).unwrap();

    // Tamper with the tag (last byte)
    let last = data.len() - 1;
    data[last] ^= 0x01;

    // Decryption should fail
    let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    assert!(auth2.decrypt(&mut data).is_err());
}

#[test]
fn test_photon_auth_version_mismatch() {
    let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    auth.encrypt(&mut data).unwrap();

    // Tamper with the version byte
    let version_pos = 64; // version byte is right after ciphertext
    data[version_pos] ^= 0x01;

    // Decryption should fail with version error
    let mut auth2 = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    let result = auth2.decrypt(&mut data);
    assert!(result.is_err(), "version mismatch should fail");
    let err = result.unwrap_err();
    match err {
        KelvinError::AuthenticationFailed(msg) => {
            assert!(msg.contains("version"), "error should mention version");
        },
        _ => panic!("expected AuthenticationFailed error"),
    }
}

#[test]
fn test_photon_auth_too_short() {
    let mut auth = KelvinPhotonAuthenticated::new(test_seed(), 1000);
    let mut data = vec![0xABu8; AUTH_OVERHEAD - 1]; // Too short for auth overhead
    assert!(auth.encrypt(&mut data).is_err());
    assert!(auth.decrypt(&mut data).is_err());
}

// ─── Quantum Authenticated Tests ───────────────────────────────────────

#[test]
fn test_quantum_auth_round_trip() {
    let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    let original = data.clone();

    auth.encrypt(&mut data).unwrap();
    assert_ne!(&data[..64], &original[..64]);

    let mut auth2 = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
    auth2.decrypt(&mut data).unwrap();
    assert_eq!(&data[..64], &original[..64]);
}

#[test]
fn test_quantum_auth_tamper_detection() {
    let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    auth.encrypt(&mut data).unwrap();

    data[0] ^= 0x01;

    let mut auth2 = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
    assert!(auth2.decrypt(&mut data).is_err());
}

#[test]
fn test_quantum_auth_too_short() {
    let mut auth = KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000);
    let mut data = vec![0xABu8; AUTH_OVERHEAD - 1];
    assert!(auth.encrypt(&mut data).is_err());
    assert!(auth.decrypt(&mut data).is_err());
}

// ─── Streaming Authenticated Tests ─────────────────────────────────────

fn streaming_config() -> OrbitalConfig {
    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54),
        Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
        Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
    );
    let planet2 = OrbitalBody::new(
        Fixed::from_raw(1 << 53),
        Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
        Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
    );
    let planet3 = OrbitalBody::new(
        Fixed::from_raw(1 << 52),
        Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
        Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
    );
    let planet4 = OrbitalBody::new(
        Fixed::from_raw(1 << 51),
        Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
        Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
    );
    OrbitalConfig::new(
        vec![sun, planet1, planet2, planet3, planet4],
        1000,
        100,
        kelvin::DEFAULT_DT,
        Fixed::from_raw(1 << 44),
        kelvin::DEFAULT_G,
    )
    .unwrap()
}

#[test]
fn test_streaming_auth_round_trip() {
    let config = streaming_config();
    let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    let original = data.clone();

    auth.encrypt(&mut data).unwrap();
    assert_ne!(&data[..64], &original[..64]);

    let mut auth2 = KelvinStreamingAuthenticated::new(streaming_config(), 64).unwrap();
    auth2.decrypt(&mut data).unwrap();
    assert_eq!(&data[..64], &original[..64]);
}

#[test]
fn test_streaming_auth_tamper_detection() {
    let config = streaming_config();
    let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
    let mut data = vec![0xABu8; 64 + AUTH_OVERHEAD];
    auth.encrypt(&mut data).unwrap();

    data[0] ^= 0x01;

    let mut auth2 = KelvinStreamingAuthenticated::new(streaming_config(), 64).unwrap();
    assert!(auth2.decrypt(&mut data).is_err());
}

#[test]
fn test_streaming_auth_too_short() {
    let config = streaming_config();
    let mut auth = KelvinStreamingAuthenticated::new(config, 64).unwrap();
    let mut data = vec![0xABu8; AUTH_OVERHEAD - 1];
    assert!(auth.encrypt(&mut data).is_err());
    assert!(auth.decrypt(&mut data).is_err());
}
