//! Full pipeline integration test.
//!
//! Tests the complete encrypt/decrypt round-trip with a 3-body system.

use kelvin::{Kelvin, OrbitalConfig, Fixed, Vec3, OrbitalBody};

fn three_body_config() -> OrbitalConfig {
    let sun = OrbitalBody::new(
        Fixed::ONE,
        Vec3::ZERO,
        Vec3::ZERO,
    );
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
    OrbitalConfig::new(
        vec![sun, planet1, planet2],
        10000,
        1000,
        Fixed::from_raw(1 << 44),
        Fixed::from_raw(1 << 44),
    ).unwrap()
}

#[test]
fn test_full_pipeline_round_trip() {
    let config = three_body_config();
    let mut k = Kelvin::new(config).unwrap();

    let mut data = b"This is a secret message from the Kelvin cryptosystem!".to_vec();
    let original = data.clone();

    k.encrypt(&mut data).unwrap();
    assert_ne!(data, original, "encrypted data should differ from original");

    k.decrypt(&mut data).unwrap();
    assert_eq!(data, original, "decrypted data should match original");
}

#[test]
fn test_multiple_blocks() {
    let config = three_body_config();
    let mut k = Kelvin::new(config).unwrap();

    let mut data = vec![0xABu8; 1024];
    let original = data.clone();

    k.encrypt(&mut data).unwrap();
    assert_ne!(data, original);

    k.decrypt(&mut data).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_empty_data() {
    let config = three_body_config();
    let mut k = Kelvin::new(config).unwrap();

    let mut data: Vec<u8> = vec![];
    k.encrypt(&mut data).unwrap();
    assert!(data.is_empty());
}

#[test]
fn test_deterministic_encryption() {
    let config1 = three_body_config();
    let config2 = three_body_config();
    let mut k1 = Kelvin::new(config1).unwrap();
    let mut k2 = Kelvin::new(config2).unwrap();

    let mut data1 = vec![0x42u8; 256];
    let mut data2 = data1.clone();

    k1.encrypt(&mut data1).unwrap();
    k2.encrypt(&mut data2).unwrap();

    assert_eq!(data1, data2, "same config should produce same keystream");
}

#[test]
fn test_bytes_processed() {
    let config = three_body_config();
    let mut k = Kelvin::new(config).unwrap();

    assert_eq!(k.bytes_processed(), 0);

    k.encrypt(&mut vec![0u8; 100]).unwrap();
    assert_eq!(k.bytes_processed(), 100);

    k.encrypt(&mut vec![0u8; 50]).unwrap();
    assert_eq!(k.bytes_processed(), 150);
}

#[test]
fn test_remaining_safe_bytes() {
    let config = three_body_config();
    let k = Kelvin::new(config).unwrap();
    assert!(k.remaining_safe_bytes() > 0);
}
