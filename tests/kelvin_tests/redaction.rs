//! `Debug` output must not reveal secret material.

use kelvin::streaming::{PhotonEncryptor, QuantumEncryptor, StreamEncrypt};
use kelvin::{
    Fixed, FlareScheme, KelvinFlare, KelvinPhotonAuthenticated, KelvinQuantum,
    KelvinQuantumAuthenticated, KelvinStreaming, KelvinStreamingAuthenticated, KeySchedule,
    OrbitalBody, OrbitalConfig, Vec3, PHOTON_BASE_SEED_SIZE,
};

/// Every byte of the test seed; its decimal form is searched for below.
const SECRET_BYTE: u8 = 0xDA;

fn seed() -> [u8; PHOTON_BASE_SEED_SIZE] {
    [SECRET_BYTE; PHOTON_BASE_SEED_SIZE]
}

fn config() -> OrbitalConfig {
    let body = |mass: i128, x: i128, vy: i128| {
        OrbitalBody::new(
            Fixed::from_raw(mass),
            Vec3::new(Fixed::from_int(x as i64), Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(vy as i64), Fixed::ZERO),
        )
    };
    OrbitalConfig::new(
        vec![
            body(1 << 64, 0, 0),
            body(1 << 54, 1, 6),
            body(1 << 53, 2, 4),
            body(1 << 52, -1, -5),
            body(1 << 51, -3, -3),
        ],
        200,
        10,
        kelvin::DEFAULT_DT,
        kelvin::SOFTENING_FACTOR,
        kelvin::DEFAULT_G,
    )
    .unwrap()
}

fn assert_redacted(value: &impl std::fmt::Debug) {
    let text = format!("{value:?}");
    // Two adjacent elements of a printed byte array, or any body field.
    for leak in ["218, 218", "position", "velocity", "mass"] {
        assert!(!text.contains(leak), "{leak:?} appears in {text}");
    }
    assert!(text.len() < 600, "unexpectedly long Debug output: {} bytes", text.len());
}

#[test]
fn config_and_schedule_are_redacted() {
    assert_redacted(&config());
    assert_redacted(&KeySchedule::new(seed(), 10_000, 1_000, 5_000));
}

#[test]
fn engines_are_redacted() {
    assert_redacted(&KelvinStreaming::new(config(), 1024).unwrap());
    assert_redacted(&KelvinQuantum::new(seed(), 10));
}

#[test]
fn authenticated_wrappers_are_redacted() {
    assert_redacted(&KelvinPhotonAuthenticated::new(seed(), 10));
    assert_redacted(&KelvinQuantumAuthenticated::new(seed(), 10));
    assert_redacted(&KelvinStreamingAuthenticated::new(config(), 1024).unwrap());
}

#[test]
fn streaming_buffers_and_keys_are_redacted() {
    let mut output = Vec::new();
    let mut photon = PhotonEncryptor::new(seed(), 10);
    photon.update(&[0u8; 64], &mut output).unwrap();
    assert_redacted(&photon);
    let mut quantum = QuantumEncryptor::new(seed(), 10);
    quantum.update(&[0u8; 64], &mut output).unwrap();
    assert_redacted(&quantum);

    let key = KelvinFlare::new(seed(), 10).generate_fhe_key(FlareScheme::Tfhe, 32).unwrap();
    let text = format!("{key:?}");
    assert!(!text.contains(&format!("{}, {}", key.key()[0], key.key()[1])), "{text}");
}
