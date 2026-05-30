//! Latency benchmark: ED25519 sign/verify (dalek).
//!
//! Measures the time to sign and verify a 1 MiB message using ED25519 (dalek).
//! HAWK is not benchmarked — no production-quality Rust HAWK implementation exists.
//! This benchmark serves as a baseline comparison point for future Kelvin
//! signature implementations.
//!
//! # Run
//! ```bash
//! cargo bench -p comparative-bench --bench signature
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

use comparative_bench::test_buffer;

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;

/// Generate a deterministic Ed25519 signing key.
fn make_signing_key() -> SigningKey {
    let mut rng = StdRng::seed_from_u64(42);
    let mut seed = [0u8; 32];
    rng.fill_bytes(&mut seed);
    SigningKey::from_bytes(&seed)
}

// ============================================================================
// ED25519 sign 1 MiB
// ============================================================================

fn bench_ed25519_sign(c: &mut Criterion) {
    let keypair = make_signing_key();
    let msg = test_buffer();

    let mut group = c.benchmark_group("ED25519 (dalek) sign");
    group.throughput(Throughput::Bytes(msg.len() as u64));
    group.bench_function("sign 1 MiB", |b| {
        b.iter(|| {
            let signature: Signature = keypair.sign(black_box(&msg));
            black_box(signature);
        })
    });
    group.finish();
}

// ============================================================================
// ED25519 verify 1 MiB
// ============================================================================

fn bench_ed25519_verify(c: &mut Criterion) {
    let keypair = make_signing_key();
    let verifying_key: VerifyingKey = (&keypair).into();
    let msg = test_buffer();
    let signature: Signature = keypair.sign(&msg);

    let mut group = c.benchmark_group("ED25519 (dalek) verify");
    group.throughput(Throughput::Bytes(msg.len() as u64));
    group.bench_function("verify 1 MiB", |b| {
        b.iter(|| {
            let result = verifying_key.verify(black_box(&msg), black_box(&signature));
            assert!(result.is_ok());
        })
    });
    group.finish();
}

// ============================================================================
// Criterion harness
// ============================================================================

criterion_group!(
    name = signature;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .sample_size(50);
    targets = bench_ed25519_sign, bench_ed25519_verify
);
criterion_main!(signature);
