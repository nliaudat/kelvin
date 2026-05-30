//! Latency benchmark: Kelvin orbital keygen vs X25519 (dalek).
//!
//! Measures the time to generate a key pair for both systems.
//! Kelvin keygen includes the full orbital simulation pipeline
//! (config validation → Lyapunov estimation → n-body simulation → seed extraction).
//! X25519 keygen is a simple scalar multiplication.
//!
//! Note: Kelvin orbital keygen is inherently slower due to the n-body simulation
//! requirement (~seconds vs microseconds). This benchmark quantifies the gap.
//!
//! # Run
//! ```bash
//! cargo bench -p comparative-bench --bench keygen
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use kelvin::OrbitalConfig;
use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};
use x25519_dalek::{PublicKey, StaticSecret};

/// Build a minimal 5-body orbital config with enough steps to pass Lyapunov
/// horizon, but fast enough for benchmarking (~500 steps).
fn keygen_orbital_config() -> OrbitalConfig {
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
    // 100000 steps with reseed_interval=1000 — passes Lyapunov for 5-body
    // (bench.py's --fast uses 110000 steps as the safe minimum)
    OrbitalConfig::new(bodies, 100_000, 1_000, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("valid config")
}

// ============================================================================
// Kelvin orbital keygen (full pipeline)
// ============================================================================

fn bench_kelvin_keygen(c: &mut Criterion) {
    let config = keygen_orbital_config();

    c.benchmark_group("Kelvin orbital keygen").bench_function(
        "full pipeline (config + simulate + extract)",
        |b| {
            b.iter(|| {
                let (seed, _bodies) =
                    kelvin::simulate_and_extract_seed(black_box(&config)).expect("keygen pipeline");
                black_box(seed);
            })
        },
    );
}

// ============================================================================
// X25519 keygen (dalek)
// ============================================================================

fn bench_x25519_keygen(c: &mut Criterion) {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    let mut rng = StdRng::seed_from_u64(42);

    c.benchmark_group("X25519 (dalek) keygen").bench_function("static secret + public key", |b| {
        b.iter(|| {
            let secret = StaticSecret::random_from_rng(&mut rng);
            let public = PublicKey::from(&secret);
            black_box((secret, public));
        })
    });
}

// ============================================================================
// Criterion harness
// ============================================================================

criterion_group!(
    name = keygen;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3))
        .sample_size(10);
    targets = bench_kelvin_keygen, bench_x25519_keygen
);
criterion_main!(keygen);
