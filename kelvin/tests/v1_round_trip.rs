//! V1 round-trip integration tests.
//!
//! Tests the complete encrypt/decrypt round-trip with a 5-body system.
//! These tests use Verlet integration (energy-conserving) to avoid body
//! ejection caused by Euler's numerical instability.

use kelvin::{Fixed, Kelvin, OrbitalBody, OrbitalConfig, Vec3};

fn test_config() -> OrbitalConfig {
    let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
    let planet1 = OrbitalBody::new(
        Fixed::from_raw(1 << 54), // ~1e-6 solar masses
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
        500, // Use enough steps to exceed Lyapunov horizon
        10,
        kelvin::DEFAULT_DT,
        Fixed::from_raw(1 << 44), // ~1e-6
        kelvin::DEFAULT_G,
    )
    .unwrap()
}

#[test]
fn test_round_trip_small() {
    let config = test_config();
    let mut k = Kelvin::new(config.clone()).unwrap();
    // Buffer needs 16 extra bytes for AEAD tag
    let mut data = vec![0xABu8; 64 + 16];
    let original = data.clone();
    k.encrypt_in_place(&mut data).unwrap();
    // Ciphertext portion (first 64 bytes) should differ from plaintext
    assert_ne!(&data[..64], &original[..64]);
    // Create a new Kelvin instance for decryption (same config = same keystream)
    let mut k2 = Kelvin::new(config).unwrap();
    k2.decrypt_in_place(&mut data).unwrap();
    // Plaintext portion should be restored; tag portion is overwritten during decrypt
    assert_eq!(&data[..64], &original[..64]);
}
