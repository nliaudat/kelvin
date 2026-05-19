//! # Kelvin V2 Streaming Example
//!
//! Demonstrates the V2 streaming cryptosystem where each chunk of data
//! advances the orbital simulation by one step.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --example simple_streaming
//! ```
//!
//! ## What it shows
//!
//! 1. Creating a `KelvinStreaming` instance
//! 2. Encrypting data one chunk at a time
//! 3. Decrypting with a new instance (same config = same keystream)
//! 4. Benchmarking simulation speed
//! 5. Estimating time for a file

use kelvin::{Fixed, KelvinStreaming, OrbitalBody, OrbitalConfig, Vec3};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Kelvin V2 Streaming Example ===\n");

    // Create a 5-body system (minimum required by validation)
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

    let config = OrbitalConfig::new(
        bodies,
        1000,
        100,
        kelvin_core::DEFAULT_DT,
        Fixed::from_raw(1 << 44),
        kelvin_core::DEFAULT_G,
    )?;

    // Benchmark simulation speed
    println!("Benchmarking simulation speed...");
    let bench_ks = KelvinStreaming::new(config.clone(), 1024)?;
    let steps_per_sec = bench_ks.benchmark(1000);
    println!("  Simulation speed: {:.0} steps/sec\n", steps_per_sec);

    // Estimate time for a 1 MB file
    let file_size = 1_048_576; // 1 MiB
    let bytes_per_step = 1024; // 1 KiB per step
    let (steps_needed, est_secs) = bench_ks.estimate_time(file_size, steps_per_sec);
    println!("To process {} bytes ({} KiB/step):", file_size, bytes_per_step);
    println!("  Steps needed: {}", steps_needed);
    println!("  Estimated time: {:.2} seconds\n", est_secs);

    // Create streaming instance
    let mut ks = KelvinStreaming::new(config.clone(), bytes_per_step)?;
    println!("Streaming instance created. Step: {}", ks.step());

    // Encrypt a message
    let message = b"Hello, Kelvin V2 Streaming! This is a test message.";
    let mut data = message.to_vec();
    println!("\nOriginal:  {:?}", std::str::from_utf8(&data).unwrap());

    ks.encrypt(&mut data)?;
    println!("Encrypted: {} bytes (step {})", data.len(), ks.step());

    // Decrypt with a new instance (same config = same keystream)
    let mut ks2 = KelvinStreaming::new(config.clone(), bytes_per_step)?;
    ks2.decrypt(&mut data)?;
    println!("\nDecrypted: {:?}", std::str::from_utf8(&data).unwrap());
    println!("Step: {}", ks2.step());

    // Verify
    assert_eq!(data, message, "Round-trip should restore original");
    println!("\n✓ Round-trip verified!");

    // Multi-chunk example
    println!("\n--- Multi-chunk Example ---");
    let mut ks3 = KelvinStreaming::new(config.clone(), 32)?; // 32 bytes per step

    let chunks = vec![
        b"Chunk 1: The quick brown fox".to_vec(),
        b"Chunk 2: jumps over the lazy".to_vec(),
        b"Chunk 3: dog near the river.".to_vec(),
    ];
    let originals = chunks.clone();

    let mut encrypted_chunks = Vec::new();
    for (i, mut chunk) in chunks.into_iter().enumerate() {
        ks3.encrypt(&mut chunk)?;
        println!("Chunk {} encrypted (step {})", i + 1, ks3.step());
        encrypted_chunks.push(chunk);
    }

    // Decrypt all chunks
    let mut ks4 = KelvinStreaming::new(config.clone(), 32)?;
    for (i, mut chunk) in encrypted_chunks.into_iter().enumerate() {
        ks4.decrypt(&mut chunk)?;
        assert_eq!(chunk, originals[i], "Chunk {} should round-trip", i + 1);
        println!("Chunk {} decrypted and verified", i + 1);
    }

    println!("\n✓ All multi-chunk tests passed!");
    println!("\nDone.");

    Ok(())
}
