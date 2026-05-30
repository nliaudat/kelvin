//! Throughput benchmark: KelvinStreaming (V2) vs AES-256-CTR.
//!
//! Measures MB/s streaming encryption throughput for 1 MiB buffers.
//! KelvinStreaming uses the ChaosEncryptor wrapper which advances the
//! n-body simulation one step per chunk and XORs with a SHAKE256 keystream.
//!
//! # Run
//! ```bash
//! cargo bench -p comparative-bench --bench throughput_streaming
//! ```

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use std::time::Duration;

use comparative_bench::{aes_ctr_key, ctr_nonce, test_buffer, BUFFER_SIZE};

use aes::Aes256;
use cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;

use kelvin::OrbitalConfig;

fn small_orbital_config() -> OrbitalConfig {
    let bodies = vec![
        kelvin::OrbitalBody::new(
            kelvin::Fixed::ONE,
            kelvin::Vec3::new(kelvin::Fixed::ZERO, kelvin::Fixed::ZERO, kelvin::Fixed::ZERO),
            kelvin::Vec3::new(kelvin::Fixed::ZERO, kelvin::Fixed::ZERO, kelvin::Fixed::ZERO),
        ),
        kelvin::OrbitalBody::new(
            kelvin::Fixed::from_raw(1 << 54),
            kelvin::Vec3::new(kelvin::Fixed::ONE, kelvin::Fixed::ZERO, kelvin::Fixed::ZERO),
            kelvin::Vec3::new(kelvin::Fixed::ZERO, kelvin::Fixed::from_int(6), kelvin::Fixed::ZERO),
        ),
        kelvin::OrbitalBody::new(
            kelvin::Fixed::from_raw(1 << 53),
            kelvin::Vec3::new(kelvin::Fixed::ZERO, kelvin::Fixed::from_int(2), kelvin::Fixed::ZERO),
            kelvin::Vec3::new(
                kelvin::Fixed::from_int(-4),
                kelvin::Fixed::ZERO,
                kelvin::Fixed::ZERO,
            ),
        ),
        kelvin::OrbitalBody::new(
            kelvin::Fixed::from_raw(1 << 52),
            kelvin::Vec3::new(
                kelvin::Fixed::from_int(-1),
                kelvin::Fixed::from_int(-1),
                kelvin::Fixed::ZERO,
            ),
            kelvin::Vec3::new(
                kelvin::Fixed::from_int(3),
                kelvin::Fixed::from_int(-2),
                kelvin::Fixed::ZERO,
            ),
        ),
        kelvin::OrbitalBody::new(
            kelvin::Fixed::from_raw(1 << 51),
            kelvin::Vec3::new(
                kelvin::Fixed::from_int(2),
                kelvin::Fixed::from_int(-1),
                kelvin::Fixed::from_int(1),
            ),
            kelvin::Vec3::new(
                kelvin::Fixed::from_int(-2),
                kelvin::Fixed::from_int(3),
                kelvin::Fixed::ZERO,
            ),
        ),
    ];
    OrbitalConfig::new(
        bodies,
        5000,
        500,
        kelvin::DEFAULT_DT,
        kelvin::SOFTENING_FACTOR,
        kelvin::DEFAULT_G,
    )
    .expect("valid config")
}

fn bench_kelvin_streaming_encrypt(c: &mut Criterion) {
    let config = small_orbital_config();

    let mut group = c.benchmark_group("KelvinStreaming (V2)");
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("encrypt 1 MiB", |b| {
        b.iter(|| {
            // Re-initialize KelvinStreaming inside the loop to prevent state
            // exhaustion (each iteration consumes one 1 MiB step).
            // KelvinStreaming::new is cheap: it only validates the config and
            // clones the body vectors — no simulation is run upfront.
            let mut data = test_buffer();
            let mut streaming = kelvin::KelvinStreaming::new(config.clone(), BUFFER_SIZE as u64)
                .expect("KelvinStreaming::new");
            streaming.encrypt(black_box(&mut data)).expect("streaming encrypt");
        })
    });
    group.finish();
}

fn bench_aes256_ctr_encrypt(c: &mut Criterion) {
    let key = aes_ctr_key();
    let nonce = ctr_nonce();

    let mut group = c.benchmark_group("AES-256-CTR");
    group.throughput(Throughput::Bytes(BUFFER_SIZE as u64));
    group.bench_function("encrypt 1 MiB", |b| {
        b.iter(|| {
            // Fresh cipher and buffer each iteration: AES-CTR is a stream cipher
            // and the same keystream would XOR to the same result on repeated
            // calls with the same key+nonce.
            let mut cipher =
                Ctr128BE::<Aes256>::new_from_slices(&key, &nonce).expect("AES-256-CTR key/nonce");
            let mut buf = test_buffer();
            cipher.apply_keystream(black_box(&mut buf));
        })
    });
    group.finish();
}

criterion_group!(
    name = throughput_streaming;
    config = Criterion::default()
        .measurement_time(Duration::from_secs(5))
        .warm_up_time(Duration::from_secs(2))
        .sample_size(50);
    targets = bench_kelvin_streaming_encrypt, bench_aes256_ctr_encrypt
);
criterion_main!(throughput_streaming);