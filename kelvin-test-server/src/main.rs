//! # Kelvin Test Vector Server
//!
//! Generates golden test vectors for cross-platform determinism verification.
//!
//! ## Usage
//!
//! ```bash
//! # Generate test vectors and save to files
//! kelvin-test-server --output ./test-vectors/
//!
//! # Generate a single test vector and print to stdout
//! kelvin-test-server --stdout
//! ```
//!
//! ## Output format
//!
//! Each test vector is a JSON file containing:
//! - `config`: The OrbitalConfig used (shared secret)
//! - `plaintext`: The input data (hex-encoded)
//! - `ciphertext`: The encrypted output (hex-encoded)
//! - `keystream`: The generated keystream (hex-encoded)
//! - `description`: Human-readable description of the test case
//!
//! ## References
//!
//! - NIST SP 800-22: Statistical testing for random number generators
//! - Bernstein, D. J. (2008). "ChaCha, a variant of Salsa20."
//!   *Workshop Record of SASC 2008*.
//! - NIST FIPS PUB 202 (2015). "SHA-3 Standard: Permutation-Based Hash
//!   and Extendable-Output Functions."

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use kelvin::Kelvin;
use kelvin_core::{Fixed, OrbitalBody, Vec3};
use serde::{Deserialize, Serialize};

/// A single test vector for cross-platform determinism verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestVector {
    /// Human-readable description.
    pub description: String,
    /// Security level name.
    pub level: String,
    /// The OrbitalConfig JSON.
    pub config_json: String,
    /// Plaintext input (hex-encoded).
    pub plaintext_hex: String,
    /// Ciphertext output (hex-encoded).
    pub ciphertext_hex: String,
    /// Keystream generated (hex-encoded).
    pub keystream_hex: String,
    /// Number of bodies.
    pub num_bodies: usize,
    /// Total simulation steps.
    pub total_steps: u64,
    /// Time taken to generate (seconds).
    pub generation_time_s: f64,
}

/// Generate a standard 5-body test configuration (Standard security level).
///
/// Uses a small step count that the Lyapunov estimator will accept.
fn standard_config() -> (Vec<OrbitalBody>, u64, u64) {
    let bodies = vec![
        // Central body (1 solar mass at origin)
        OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO),
        // Planet 1: at 1 AU, circular orbit
        OrbitalBody::new(
            Fixed::from_raw(1 << 54), // ~1e-6 solar masses
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        ),
        // Planet 2: at 1.5 AU, slightly eccentric
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(
                Fixed::from_raw(3 << 63), // 1.5 AU
                Fixed::ZERO,
                Fixed::ZERO,
            ),
            Vec3::new(
                Fixed::ZERO,
                Fixed::from_raw(4896710557980672i128), // ~5 AU/yr
                Fixed::from_raw(1 << 62),              // slight z-component
            ),
        ),
        // Planet 3: at -1 AU, retrograde-ish
        OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        ),
        // Planet 4: at 2 AU, inclined
        OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
        ),
    ];
    let total_steps = 50;
    let reseed_interval = 10;
    (bodies, total_steps, reseed_interval)
}

/// Generate a 5-body test configuration (Paranoid security level).
fn paranoid_config() -> (Vec<OrbitalBody>, u64, u64) {
    let bodies = vec![
        // Central body
        OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO),
        // Planet 1: inner orbit
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::from_raw(3 << 62), Fixed::ZERO, Fixed::ZERO), // 0.75 AU
            Vec3::new(Fixed::ZERO, Fixed::from_int(7), Fixed::ZERO),
        ),
        // Planet 2: at 1 AU
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        ),
        // Planet 3: at 1.8 AU, inclined
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(
                Fixed::from_raw(4611686018427387904i128), // ~2.0 AU
                Fixed::ZERO,
                Fixed::ZERO,
            ),
            Vec3::new(
                Fixed::ZERO,
                Fixed::from_int(4),
                Fixed::from_raw(1 << 62),
            ),
        ),
        // Planet 4: outer, retrograde
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(
                Fixed::from_raw(6 << 63), // 3.0 AU
                Fixed::ZERO,
                Fixed::ZERO,
            ),
            Vec3::new(
                Fixed::ZERO,
                Fixed::from_int(-3),
                Fixed::ZERO,
            ),
        ),
    ];
    let total_steps = 50;
    let reseed_interval = 10;
    (bodies, total_steps, reseed_interval)
}

/// Generate test vectors for a given configuration.
fn generate_test_vector(
    description: &str,
    level: &str,
    bodies: Vec<OrbitalBody>,
    total_steps: u64,
    reseed_interval: u64,
    plaintext: &[u8],
) -> Result<TestVector, String> {
    let start = Instant::now();

    // Create config
    let config = kelvin::OrbitalConfig::new(
        bodies.clone(),
        total_steps,
        reseed_interval,
        Fixed::from_raw(1 << 44), // dt ~ 1e-6 years
        Fixed::from_raw(1 << 44), // softening ~ 1e-6 AU
        kelvin_core::DEFAULT_G,
    ).map_err(|e| format!("Config error: {}", e))?;

    let config_json = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("JSON error: {}", e))?;

    // Create Kelvin instance and encrypt
    let mut k = Kelvin::new(config).map_err(|e| format!("Kelvin error: {}", e))?;
    let mut data = plaintext.to_vec();
    k.encrypt(&mut data).map_err(|e| format!("Encrypt error: {}", e))?;

    let elapsed = start.elapsed().as_secs_f64();

    Ok(TestVector {
        description: description.to_string(),
        level: level.to_string(),
        config_json,
        plaintext_hex: hex::encode(plaintext),
        ciphertext_hex: hex::encode(&data),
        keystream_hex: hex::encode(
            // XOR plaintext with ciphertext to recover keystream
            plaintext
                .iter()
                .zip(data.iter())
                .map(|(p, c)| p ^ c)
                .collect::<Vec<_>>(),
        ),
        num_bodies: bodies.len(),
        total_steps,
        generation_time_s: elapsed,
    })
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();

    let output_dir = if args.len() > 1 && args[1] == "--output" {
        args.get(2).map(PathBuf::from)
    } else {
        None
    };

    let to_stdout = args.contains(&"--stdout".to_string());

    println!("Kelvin Test Vector Server");
    println!("=========================");
    println!();

    // Test data
    let plaintext_short = b"Hello, Kelvin! This is a test of the orbital chaos KDF cryptosystem.";
    let plaintext_empty: &[u8] = b"";
    let plaintext_1k = vec![0xABu8; 1024];

    // Generate test vectors
    let mut vectors = Vec::new();

    // Standard level tests
    println!("Generating Standard-level test vectors...");
    let (bodies, steps, reseed) = standard_config();

    vectors.push(generate_test_vector(
        "Standard level, short plaintext (5 bodies)",
        "Standard",
        bodies.clone(),
        steps,
        reseed,
        plaintext_short,
    )?);

    vectors.push(generate_test_vector(
        "Standard level, empty plaintext (5 bodies)",
        "Standard",
        bodies.clone(),
        steps,
        reseed,
        plaintext_empty,
    )?);

    vectors.push(generate_test_vector(
        "Standard level, 1KB plaintext (5 bodies)",
        "Standard",
        bodies.clone(),
        steps,
        reseed,
        &plaintext_1k,
    )?);

    // Paranoid level tests
    println!("Generating Paranoid-level test vectors...");
    let (bodies5, steps5, reseed5) = paranoid_config();

    vectors.push(generate_test_vector(
        "Paranoid level, short plaintext (5 bodies)",
        "Paranoid",
        bodies5.clone(),
        steps5,
        reseed5,
        plaintext_short,
    )?);

    vectors.push(generate_test_vector(
        "Paranoid level, 1KB plaintext (5 bodies)",
        "Paranoid",
        bodies5.clone(),
        steps5,
        reseed5,
        &plaintext_1k,
    )?);

    // Summary
    println!();
    println!("Generated {} test vectors:", vectors.len());
    for v in &vectors {
        println!(
            "  [{:>8}] {:>3} bodies, {:>5} steps, {:>8.3}s -- {}",
            v.level, v.num_bodies, v.total_steps, v.generation_time_s, v.description
        );
    }

    // Output
    if to_stdout {
        println!();
        println!("=== TEST VECTORS (JSON) ===");
        for v in &vectors {
            println!("{}", serde_json::to_string_pretty(v).map_err(|e| format!("JSON error: {}", e))?);
            println!("---");
        }
    }

    if let Some(dir) = output_dir {
        fs::create_dir_all(&dir).map_err(|e| format!("FS error: {}", e))?;
        for (i, v) in vectors.iter().enumerate() {
            let filename = format!(
                "kelvin-test-{}-{}.json",
                v.level.to_lowercase(),
                i + 1
            );
            let path = dir.join(&filename);
            let json = serde_json::to_string_pretty(v).map_err(|e| format!("JSON error: {}", e))?;
            fs::write(&path, json).map_err(|e| format!("FS error: {}", e))?;
            println!("  Wrote: {}", path.display());
        }
    }

    // Verification summary
    println!();
    println!("To verify on another platform:");
    println!("  kelvin-test-client --vectors <dir>");
    println!();
    println!("All test vectors generated successfully.");

    Ok(())
}
