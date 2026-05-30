//! Throughput benchmark: KelvinQuantum (H) vs AES-256-GCM vs ChaCha20-Poly1305.
//!
//! Measures MB/s encryption throughput for 1 MiB buffers.
//! All comparisons use the same buffer size and measure encrypt-only time
//! (excluding authentication tag verification for AEAD modes).
//!
//! # Run
//! ```bash
//! cargo bench -p comparative-bench --bench throughput_quantum
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

use comparative_bench::{aes_key, dummy_seed_2048, test_buffer, BUFFER_SIZE};
use rand::rngs::StdRng;
use rand::RngCore;
use rand::SeedableRng;

/// Create a deterministic 12-byte nonce for ring AEAD.
fn nonce_12() -> [u8; 12] {
    let mut rng = StdRng::seed_from_u64(7);
    let mut nonce = [0u8; 12];
    rng.fill_bytes(&mut nonce);
    nonce
}

// ============================================================================
// KelvinQuantum Encrypt
// ============================================================================

fn bench_kelvin_quantum_encrypt(c: &mut Criterion) {
    let seed = dummy_seed_2048();
    let mut buf = test_buffer();

    let mut quantum = kelvin::KelvinQuantum::new(seed, 100_000);

    let mut group = c.benchmark_group("KelvinQuantum (H)");
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("encrypt 1 MiB", |b| {
        b.iter(|| {
            quantum.encrypt(black_box(&mut buf)).expect("quantum encrypt");
        })
    });
    group.finish();
}

// ============================================================================
// AES-256-GCM (ring) Encrypt
// ============================================================================

fn bench_aes256_gcm_encrypt(c: &mut Criterion) {
    let key_bytes = aes_key();
    let buf = test_buffer();
    let nonce_bytes = nonce_12();

    let mut group = c.benchmark_group("AES-256-GCM (ring)");
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("encrypt 1 MiB", |b| {
        b.iter(|| {
            let key = ring::aead::UnboundKey::new(&ring::aead::AES_256_GCM, &key_bytes)
                .expect("valid AES-256-GCM key");
            let nonce = ring::aead::Nonce::assume_unique_for_key(nonce_bytes);
            let sealing_key = ring::aead::LessSafeKey::new(key);

            let mut in_out = buf.clone();
            in_out.extend_from_slice(&[0u8; 16]); // tag space

            let _ = sealing_key.seal_in_place_separate_tag(
                nonce,
                ring::aead::Aad::empty(),
                &mut in_out,
            );
        })
    });
    group.finish();
}

// ============================================================================
// ChaCha20-Poly1305 (ring) Encrypt
// ============================================================================

fn bench_chacha20_poly1305_encrypt(c: &mut Criterion) {
    let key_bytes = aes_key();
    let buf = test_buffer();
    let nonce_bytes = nonce_12();

    let mut group = c.benchmark_group("ChaCha20-Poly1305 (ring)");
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("encrypt 1 MiB", |b| {
        b.iter(|| {
            let key = ring::aead::UnboundKey::new(&ring::aead::CHACHA20_POLY1305, &key_bytes)
                .expect("valid ChaCha20-Poly1305 key");
            let nonce = ring::aead::Nonce::assume_unique_for_key(nonce_bytes);
            let sealing_key = ring::aead::LessSafeKey::new(key);

            let mut in_out = buf.clone();
            in_out.extend_from_slice(&[0u8; 16]); // tag space

            let _ = sealing_key.seal_in_place_separate_tag(
                nonce,
                ring::aead::Aad::empty(),
                &mut in_out,
            );
        })
    });
    group.finish();
}

// ============================================================================
// Criterion harness
// ============================================================================

criterion_group!(
    name = throughput_quantum;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .sample_size(50);
    targets = bench_kelvin_quantum_encrypt, bench_aes256_gcm_encrypt, bench_chacha20_poly1305_encrypt
);
criterion_main!(throughput_quantum);
