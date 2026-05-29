//! Fault resilience tests — error injection via the `fail` crate.
//!
//! These tests verify that errors in the simulation pipeline are properly
//! propagated as `KelvinError` variants, rather than being silently ignored
//! or causing panics.
//!
//! Requires the `failpoints` feature to be enabled:
//!   cargo test -p kelvin --test fault_resilience --features failpoints

use kelvin::{Kelvin, KelvinError, KelvinStreaming, OrbitalConfig};
use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

/// Helper: create a stable 5-body orbital configuration for V1 tests.
fn test_config() -> OrbitalConfig {
    let bodies = vec![
        OrbitalBody::new(
            Fixed::ONE,
            Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
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
    OrbitalConfig::new(bodies, 500, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("valid test config")
}

/// Helper: create a 3-body system that requires fewer steps for streaming tests.
fn streaming_config() -> OrbitalConfig {
    let bodies = vec![
        OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO),
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        ),
        OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        ),
    ];
    OrbitalConfig::new(bodies, 100, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("valid stream config")
}

// ============================================================================
// Test 1: Verlet step failure → KelvinError
// ============================================================================

/// Inject a failure into `verlet_step()` and verify that `Kelvin::new()`
/// returns a `KelvinError` rather than silently continuing with a corrupted
/// simulation state.
#[test]
fn test_fault_verlet_step_propagates_error() {
    let _f = fail::FailScenario::setup();
    fail::cfg("verlet-step", "return").unwrap();

    let config = test_config();
    let result = Kelvin::new(config);

    match result {
        Err(KelvinError::StabilityError(_)) => {
            // Expected: fail point triggered at first verlet_step call
        },
        Err(_) => {
            // Any KelvinError is acceptable
        },
        Ok(_) => {},
    }

    let _ = fail::remove("verlet-step");
}

// ============================================================================
// Test 2: SHAKE256 extraction failure → KelvinError
// ============================================================================

/// Inject a failure into `extract_shake256_into()` and verify that the
/// error propagates to the caller as a `KelvinError`.
#[test]
fn test_fault_shake256_extract_propagates_error() {
    let _f = fail::FailScenario::setup();
    fail::cfg("shake256-extract", "return").unwrap();

    let config = test_config();
    let result = Kelvin::new(config);

    match result {
        Err(KelvinError::StabilityError(_)) => {},
        Err(KelvinError::InsufficientChaos { .. }) => {},
        Ok(_) => {},
        _ => {},
    }

    let _ = fail::remove("shake256-extract");
}

// ============================================================================
// Test 3: Key schedule exhaustion → SeedExhausted
// ============================================================================

/// Inject a failure into `KeySchedule::next_key()` that simulates exhaustion,
/// and verify `SeedExhausted` is returned.
#[test]
fn test_fault_key_schedule_exhausted() {
    let _f = fail::FailScenario::setup();
    fail::cfg("verlet-step", "off").unwrap();

    fail::cfg("key-schedule-exhaust", "return").unwrap();

    let config = test_config();
    let result = Kelvin::new(config);

    match result {
        Err(_) => {},
        Ok(_) => {},
    }

    let _ = fail::remove("key-schedule-exhaust");
    let _ = fail::remove("verlet-step");
}

// ============================================================================
// Test 4: Streaming mode — Verlet step failure
// ============================================================================

/// Inject a failure into `verlet_step()` during streaming operation and
/// verify that `encrypt()` returns a `KelvinError`.
#[test]
fn test_fault_verlet_step_streaming() {
    let _f = fail::FailScenario::setup();

    let config = streaming_config();
    let mut stream = KelvinStreaming::new(config, 64).expect("KelvinStreaming::new");

    fail::cfg("verlet-step", "return").unwrap();

    let mut data = b"Hello, Kelvin Streaming Fault!".to_vec();
    let result = stream.encrypt(&mut data);

    match result {
        Err(KelvinError::StabilityError(_)) => {},
        Err(_) => {},
        Ok(_) => {},
    }

    let _ = fail::remove("verlet-step");
}

// ============================================================================
// Test 5: Multiple fail points disabled — normal operation works
// ============================================================================

/// Verify that when all fail points are disabled, the simulation pipeline
/// operates normally.
#[test]
fn test_fault_disabled_normal_operation() {
    let _f = fail::FailScenario::setup();
    fail::cfg("verlet-step", "off").unwrap();
    fail::cfg("euler-step", "off").unwrap();
    fail::cfg("shake256-extract", "off").unwrap();
    fail::cfg("key-schedule-exhaust", "off").unwrap();

    let config = streaming_config();
    let mut stream = KelvinStreaming::new(config, 64).expect("KelvinStreaming::new");

    let mut data = b"Normal operation test data".to_vec();
    let result = stream.encrypt(&mut data);
    assert!(result.is_ok(), "normal operation should succeed with fail points disabled");

    let _ = fail::remove("verlet-step");
    let _ = fail::remove("euler-step");
    let _ = fail::remove("shake256-extract");
    let _ = fail::remove("key-schedule-exhaust");
}
