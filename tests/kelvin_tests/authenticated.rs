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

// ─── Chunk order and truncation ────────────────────────────────────────

/// The same ordering and truncation checks for each authenticated wrapper.
macro_rules! chunk_binding_tests {
    ($module:ident, $wrapper:ty, $new:expr) => {
        mod $module {
            use super::*;

            fn sealed(sender: &mut $wrapper, fill: u8, last: bool) -> Vec<u8> {
                let mut chunk = vec![fill; 64 + AUTH_OVERHEAD];
                if last {
                    sender.encrypt_final(&mut chunk).unwrap();
                } else {
                    sender.encrypt(&mut chunk).unwrap();
                }
                chunk
            }

            #[test]
            fn multi_chunk_round_trip_with_empty_final_chunk() {
                let (mut sender, mut receiver): ($wrapper, $wrapper) = ($new, $new);
                let mut first = sealed(&mut sender, 1, false);
                let mut second = sealed(&mut sender, 2, false);
                let mut end = vec![0u8; AUTH_OVERHEAD];
                sender.encrypt_final(&mut end).unwrap();

                receiver.decrypt(&mut first).unwrap();
                receiver.decrypt(&mut second).unwrap();
                receiver.decrypt_final(&mut end).unwrap();
                assert_eq!(&first[..64], &[1u8; 64]);
                assert_eq!(&second[..64], &[2u8; 64]);
            }

            #[test]
            fn reordered_chunk_is_rejected_without_advancing() {
                let (mut sender, mut receiver): ($wrapper, $wrapper) = ($new, $new);
                let mut first = sealed(&mut sender, 1, false);
                let mut second = sealed(&mut sender, 2, true);

                let untouched = second.clone();
                assert!(receiver.decrypt_final(&mut second).is_err());
                assert_eq!(second, untouched);

                receiver.decrypt(&mut first).unwrap();
                receiver.decrypt_final(&mut second).unwrap();
                assert_eq!(&second[..64], &[2u8; 64]);
            }

            #[test]
            fn repeated_chunk_is_rejected() {
                let (mut sender, mut receiver): ($wrapper, $wrapper) = ($new, $new);
                let first = sealed(&mut sender, 1, false);

                receiver.decrypt(&mut first.clone()).unwrap();
                assert!(receiver.decrypt(&mut first.clone()).is_err());
            }

            #[test]
            fn truncated_stream_is_rejected() {
                let (mut sender, mut receiver): ($wrapper, $wrapper) = ($new, $new);
                let first = sealed(&mut sender, 1, false);

                // The last chunk received was not sealed as final.
                assert!(receiver.decrypt_final(&mut first.clone()).is_err());
            }

            #[test]
            fn final_chunk_is_not_accepted_as_a_middle_chunk() {
                let (mut sender, mut receiver): ($wrapper, $wrapper) = ($new, $new);
                let mut only = sealed(&mut sender, 1, true);

                assert!(receiver.decrypt(&mut only).is_err());
            }

            #[test]
            fn nothing_follows_the_final_chunk() {
                let mut sender: $wrapper = $new;
                sealed(&mut sender, 1, true);

                let mut extra = vec![0u8; 64 + AUTH_OVERHEAD];
                assert!(sender.encrypt(&mut extra).is_err());
                assert!(sender.encrypt_final(&mut extra).is_err());
            }
        }
    };
}

#[test]
fn test_streaming_auth_rejects_mismatched_settings() {
    use kelvin::IntegrationMethod::{Euler, Verlet};
    let new = |bytes_per_step, method| {
        KelvinStreamingAuthenticated::new_with_method_and_nonce(
            streaming_config(),
            bytes_per_step,
            method,
            b"nonce",
        )
        .unwrap()
    };
    let mut chunk = vec![7u8; 64 + AUTH_OVERHEAD];
    new(1024, Euler).encrypt_final(&mut chunk).unwrap();

    for mut receiver in [new(1024, Verlet), new(512, Euler)] {
        assert!(receiver.decrypt_final(&mut chunk.clone()).is_err());
    }
    new(1024, Euler).decrypt_final(&mut chunk).unwrap();
    assert_eq!(&chunk[..64], &[7u8; 64]);
}

chunk_binding_tests!(
    photon_chunks,
    KelvinPhotonAuthenticated,
    KelvinPhotonAuthenticated::new(test_seed(), 1000)
);
chunk_binding_tests!(
    quantum_chunks,
    KelvinQuantumAuthenticated,
    KelvinQuantumAuthenticated::new(test_quantum_seed(), 1000)
);
chunk_binding_tests!(
    streaming_chunks,
    KelvinStreamingAuthenticated,
    KelvinStreamingAuthenticated::new(streaming_config(), 1024).unwrap()
);
