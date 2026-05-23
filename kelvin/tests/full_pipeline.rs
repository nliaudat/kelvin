//! Full pipeline integration test.
//!
//! Tests the complete encrypt/decrypt round-trip with a 5-body system.
//!
//! These V1 tests use Verlet integration (energy-conserving) to avoid body
//! ejection caused by Euler's numerical instability. The V1 pipeline tests
//! AEAD round-trips, not chaos properties — Verlet is the right choice here.

use kelvin::{Fixed, IntegrationMethod, Kelvin, KelvinStreaming, OrbitalBody, OrbitalConfig, Vec3};

fn five_body_config() -> OrbitalConfig {
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
        10000,
        1000,
        kelvin_core::DEFAULT_DT,
        Fixed::from_raw(1 << 44),
        kelvin_core::DEFAULT_G,
    )
    .unwrap()
}

/// Create a Kelvin instance using Verlet integration for V1 pipeline tests.
///
/// Verlet's energy conservation keeps the 5-body system stable for the full
/// 10000 steps. Euler's numerical instability causes body 3 ejection at
/// ~1500 steps, which would break these AEAD round-trip tests.
fn make_kelvin(config: OrbitalConfig) -> Kelvin {
    Kelvin::new_with_method(config, IntegrationMethod::Verlet).unwrap()
}

#[test]
fn test_full_pipeline_round_trip() {
    let config = five_body_config();
    let mut enc = make_kelvin(config.clone());
    let mut dec = make_kelvin(config);

    let plaintext = b"This is a secret message from the Kelvin cryptosystem!";
    let mut data = plaintext.to_vec();
    data.extend_from_slice(&[0u8; 16]);
    let original = data.clone();

    enc.encrypt(&mut data).unwrap();
    assert_ne!(
        &data[..plaintext.len()],
        &original[..plaintext.len()],
        "encrypted data should differ from original"
    );

    dec.decrypt(&mut data).unwrap();
    assert_eq!(
        &data[..plaintext.len()],
        &original[..plaintext.len()],
        "decrypted data should match original"
    );
}

#[test]
fn test_multiple_blocks() {
    let config = five_body_config();
    let mut enc = make_kelvin(config.clone());
    let mut dec = make_kelvin(config);

    let mut data = vec![0xABu8; 1024 + 16];
    let original = data.clone();

    enc.encrypt(&mut data).unwrap();
    assert_ne!(&data[..1024], &original[..1024]);

    dec.decrypt(&mut data).unwrap();
    assert_eq!(&data[..1024], &original[..1024]);
}

#[test]
fn test_empty_data() {
    let config = five_body_config();
    let mut k = make_kelvin(config);

    let mut data: Vec<u8> = vec![0u8; 16];
    k.encrypt(&mut data).unwrap();
    assert_eq!(data.len(), 16);
}

#[test]
fn test_deterministic_encryption() {
    let config1 = five_body_config();
    let config2 = five_body_config();
    let mut k1 = make_kelvin(config1);
    let mut k2 = make_kelvin(config2);

    let mut data1 = vec![0x42u8; 256 + 16];
    let mut data2 = data1.clone();

    k1.encrypt(&mut data1).unwrap();
    k2.encrypt(&mut data2).unwrap();

    assert_eq!(data1, data2, "same config should produce same keystream");
}

#[test]
fn test_bytes_processed() {
    let config = five_body_config();
    let mut k = make_kelvin(config);

    assert_eq!(k.bytes_processed(), 0);

    k.encrypt(&mut [0u8; 84 + 16]).unwrap();
    assert_eq!(k.bytes_processed(), 84);

    k.encrypt(&mut [0u8; 34 + 16]).unwrap();
    assert_eq!(k.bytes_processed(), 118);
}

#[test]
fn test_remaining_safe_bytes() {
    let config = five_body_config();
    let k = make_kelvin(config);
    let remaining = k.remaining_safe_bytes();
    assert_eq!(
        remaining % (1u64 << 32),
        0,
        "remaining_safe_bytes should be a multiple of 4 GiB, got {}",
        remaining
    );
}

#[test]
fn test_aead_tag_detection() {
    let config = five_body_config();
    let mut enc = make_kelvin(config.clone());

    let mut data = vec![0xABu8; 64 + 16];
    enc.encrypt(&mut data).unwrap();

    data[10] ^= 0xFF;

    let mut dec = make_kelvin(config);
    let result = dec.decrypt(&mut data);
    assert!(result.is_err(), "AEAD must detect tampered ciphertext");
}

#[test]
fn test_key_rotation() {
    let config = five_body_config();
    let mut k = make_kelvin(config);

    for i in 0..100 {
        let mut msg = vec![i as u8; 16 + 16];
        k.encrypt(&mut msg).unwrap();
    }

    assert_eq!(k.bytes_processed(), 100 * 16);
}

// --- V2 Streaming Integration Tests ---

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
        kelvin_core::DEFAULT_DT,
        Fixed::from_raw(1 << 44),
        kelvin_core::DEFAULT_G,
    )
    .unwrap()
}

#[test]
fn test_streaming_round_trip() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 64).unwrap();
    let mut data = b"Hello, Kelvin V2 streaming!".to_vec();
    let original = data.clone();

    ks.encrypt(&mut data).unwrap();
    assert_ne!(data, original, "encrypted data should differ from plaintext");

    let mut ks2 = KelvinStreaming::new(streaming_config(), 64).unwrap();
    ks2.decrypt(&mut data).unwrap();
    assert_eq!(data, original, "round-trip should restore original");
}

#[test]
fn test_streaming_determinism() {
    let config = streaming_config();
    let mut ks1 = KelvinStreaming::new(config.clone(), 64).unwrap();
    let mut ks2 = KelvinStreaming::new(config, 64).unwrap();

    let mut data1 = b"Determinism test data".to_vec();
    let mut data2 = data1.clone();

    ks1.encrypt(&mut data1).unwrap();
    ks2.encrypt(&mut data2).unwrap();
    assert_eq!(data1, data2, "two instances should produce identical ciphertext");
}

#[test]
fn test_streaming_multi_chunk() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 32).unwrap();

    let chunk1 = b"First chunk of data!".to_vec();
    let chunk2 = b"Second chunk, different.".to_vec();
    let orig1 = chunk1.clone();
    let orig2 = chunk2.clone();

    let mut c1 = chunk1;
    let mut c2 = chunk2;

    ks.encrypt(&mut c1).unwrap();
    ks.encrypt(&mut c2).unwrap();

    assert_ne!(c1, orig1);
    assert_ne!(c2, orig2);

    let mut ks2 = KelvinStreaming::new(streaming_config(), 32).unwrap();
    ks2.decrypt(&mut c1).unwrap();
    ks2.decrypt(&mut c2).unwrap();
    assert_eq!(c1, orig1);
    assert_eq!(c2, orig2);
}

#[test]
fn test_streaming_large_data() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 1024).unwrap();

    let mut data = vec![0x42u8; 10 * 1024];
    let original = data.clone();

    ks.encrypt(&mut data).unwrap();
    assert_ne!(data, original);

    let mut ks2 = KelvinStreaming::new(streaming_config(), 1024).unwrap();
    ks2.decrypt(&mut data).unwrap();
    assert_eq!(data, original);
}

#[test]
fn test_streaming_empty_data() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 64).unwrap();
    let mut empty: Vec<u8> = vec![];
    ks.encrypt(&mut empty).unwrap();
    assert!(empty.is_empty());
}

#[test]
fn test_streaming_step_counter() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 64).unwrap();
    assert_eq!(ks.step(), 0);

    let mut data = vec![0u8; 64];
    ks.encrypt(&mut data).unwrap();
    assert_eq!(ks.step(), 1);

    ks.encrypt(&mut data).unwrap();
    assert_eq!(ks.step(), 2);
}

#[test]
fn test_streaming_bytes_processed() {
    let config = streaming_config();
    let mut ks = KelvinStreaming::new(config, 64).unwrap();
    assert_eq!(ks.bytes_processed(), 0);

    let mut data = vec![0u8; 100];
    ks.encrypt(&mut data).unwrap();
    assert_eq!(ks.bytes_processed(), 100);

    let mut data2 = vec![0u8; 50];
    ks.encrypt(&mut data2).unwrap();
    assert_eq!(ks.bytes_processed(), 150);
}

#[test]
fn test_streaming_benchmark() {
    let config = streaming_config();
    let ks = KelvinStreaming::new(config, 64).unwrap();
    let rate = ks.benchmark(100);
    assert!(rate > 0.0, "benchmark should return positive rate");
}
