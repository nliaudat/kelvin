//! Performance benchmarks for the Kelvin cryptosystem.
//!
//! Runs setup time and encryption throughput benchmarks across all
//! security levels (standard, paranoid, maximum) using fast mode
//! for quick iteration.

use anyhow::Result;
use kelvin::{Kelvin, DEFAULT_BYTES_PER_STEP};

use crate::keygen::generate_config;

/// Run all benchmarks.
pub fn run_benchmark() -> Result<()> {
    println!("Running Kelvin Benchmarks...");
    let levels = ["standard", "paranoid", "maximum"];

    for level in levels {
        println!("\nLevel: {}", level);
        // Use fast mode for benchmarking to avoid long simulation times
        let config = generate_config(level, true)?;
        let start = std::time::Instant::now();
        let _ = Kelvin::new(config)?;
        let duration = start.elapsed();
        println!("  Setup Time: {:?}", duration);

        let mut data = vec![0u8; DEFAULT_BYTES_PER_STEP as usize];
        let mut k = Kelvin::new(generate_config(level, true)?)?;
        let start = std::time::Instant::now();
        k.encrypt(&mut data)?;
        let duration = start.elapsed();
        let secs = duration.as_secs_f64();
        let throughput = if secs > 0.0 { (data.len() as f64 / 1_000_000.0) / secs } else { 0.0 };
        println!("  Encryption Throughput (ChaCha20): {:.2} MB/s", throughput);
    }

    Ok(())
}
