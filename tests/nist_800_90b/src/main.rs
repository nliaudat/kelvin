// Test/bench crates are allowed to use f64 math for statistical analysis.
// The clippy config disallows f64 methods for cryptographic code (which uses
// Fixed-point), but entropy analysis inherently needs floating-point math.
#![allow(clippy::disallowed_methods)]

//! # NIST SP 800-90B Entropy Source Validation — Kelvin Cryptosystem
//!
//! This tool generates raw keystream from the Kelvin orbital chaos KDF for
//! analysis with the official NIST SP 800-90B entropy assessment tool (`ea_iid`).
//!
//! ## Usage
//!
//! ```bash
//! # Generate 1 GB of raw keystream for NIST ea_iid analysis
//! cargo run -p nist_800_90b -- generate --size 1073741824 --output keystream.bin
//!
//! # Use a custom orbital configuration
//! cargo run -p nist_800_90b -- generate --size 1073741824 --output keystream.bin --config my_config.json
//!
//! # Run the built-in SP 800-90B health tests on a keystream file
//! cargo run -p nist_800_90b -- analyze --input keystream.bin
//! ```
//!
//! ## NIST ea_iid Tool
//!
//! After generating the keystream, run the official NIST tool:
//!
//! ```bash
//! python ea_iid.py -i keystream.bin -o results.txt
//! ```
//!
//! The NIST tool is available from:
//! https://github.com/usnistgov/SP800-90B_EntropyAssessment
//!
//! ## What is "Un-conditioned" Keystream?
//!
//! The raw keystream is generated using `KelvinStreaming` (V2 mode), which
//! produces SHAKE256 XOR output directly from the orbital simulation state.
//! This is the rawest form of keystream before any cipher layer (ChaCha20,
//! AEAD, HKDF) is applied. The conditioning component (per NIST SP 800-90C)
//! is SHAKE256 itself, a NIST-approved XOF.

use std::fs;
use std::io::{Read, Write};
use std::path::Path;

use kelvin::{Fixed, IntegrationMethod, KelvinStreaming, OrbitalBody, OrbitalConfig, Vec3};
use kelvin_core::{DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

// ── CLI argument parsing (no clap dependency) ─────────────────────────────────

struct Args {
    command: Command,
}

enum Command {
    Generate { size: u64, output: String, config: Option<String>, verlet: bool },
    Analyze { input: String },
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage:");
        eprintln!(
            "  nist_800_90b generate --size <BYTES> --output <FILE> [--config <JSON>] [--verlet]"
        );
        eprintln!("  nist_800_90b analyze --input <FILE>");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "generate" => {
            let mut size: u64 = 1024 * 1024; // default 1 MB
            let mut output = String::from("keystream.bin");
            let mut config: Option<String> = None;
            let mut verlet = false;

            let mut i = 2;
            while i < args.len() {
                match args[i].as_str() {
                    "--size" => {
                        i += 1;
                        if i < args.len() {
                            size = args[i].parse().unwrap_or(1024 * 1024);
                        }
                    },
                    "--output" => {
                        i += 1;
                        if i < args.len() {
                            output = args[i].clone();
                        }
                    },
                    "--config" => {
                        i += 1;
                        if i < args.len() {
                            config = Some(args[i].clone());
                        }
                    },
                    "--verlet" => verlet = true,
                    _ => {},
                }
                i += 1;
            }

            Args { command: Command::Generate { size, output, config, verlet } }
        },
        "analyze" => {
            let mut input = String::from("keystream.bin");
            let mut i = 2;
            while i < args.len() {
                if args[i].as_str() == "--input" {
                    i += 1;
                    if i < args.len() {
                        input = args[i].clone();
                    }
                }
                i += 1;
            }
            Args { command: Command::Analyze { input } }
        },
        _ => {
            eprintln!("Unknown command: {}. Use 'generate' or 'analyze'.", args[1]);
            std::process::exit(1);
        },
    }
}

// ── Default 5-body orbital configuration ──────────────────────────────────────
//
// This is a fixed, deterministic configuration so that keystream generation
// is reproducible across runs. For NIST validation, use the default config
// to establish a baseline, then optionally test with custom configs.

fn default_config() -> OrbitalConfig {
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
        2000,
        10,
        DEFAULT_DT,
        SOFTENING_FACTOR,
        DEFAULT_G,
    )
    .expect("Default config should be valid")
}

// ── Keystream generation ──────────────────────────────────────────────────────
//
// Uses KelvinStreaming (V2) to produce raw SHAKE256 XOR keystream.
// This is the "un-conditioned" output before any cipher layer.
// The conditioning component (SHAKE256) is documented per NIST SP 800-90C.

fn generate_keystream(
    config: OrbitalConfig,
    size: u64,
    method: IntegrationMethod,
    output_path: &str,
) -> Result<(), String> {
    // Use a large bytes_per_step to minimize simulation steps
    // 64 KB per step is a good balance between speed and memory
    let bytes_per_step = 64 * 1024; // 64 KB per simulation step

    let mut ks = KelvinStreaming::new_with_method(config, bytes_per_step, method)
        .map_err(|e| format!("Failed to initialize KelvinStreaming: {:?}", e))?;

    // Benchmark to estimate time
    let rate = ks.benchmark(100);
    let (steps_needed, est_secs) = ks.estimate_time(size, rate);
    eprintln!("Estimated: {} steps, ~{:.1}s ({:.0} steps/sec)", steps_needed, est_secs, rate);

    // Create output file
    let mut output_file = fs::File::create(output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;

    // Generate keystream in chunks
    let chunk_size = bytes_per_step as usize;
    let mut buffer = vec![0u8; chunk_size];
    let mut total_written: u64 = 0;
    let mut last_progress: u64 = 0;
    let progress_interval = size / 100; // 1% progress updates

    eprintln!("Generating keystream...");
    while total_written < size {
        let remaining = (size - total_written) as usize;
        let write_size = std::cmp::min(remaining, chunk_size);

        // Fill buffer with zeros (plaintext), then encrypt to get keystream
        buffer[..write_size].fill(0);
        ks.encrypt(&mut buffer[..write_size])
            .map_err(|e| format!("Encryption failed at byte {}: {:?}", total_written, e))?;

        // Write keystream to disk
        output_file
            .write_all(&buffer[..write_size])
            .map_err(|e| format!("Write failed at byte {}: {}", total_written, e))?;

        total_written += write_size as u64;

        // Progress indicator
        if total_written - last_progress >= progress_interval {
            let pct = total_written as f64 / size as f64 * 100.0;
            eprint!("\r  Progress: {:.1}% ({}/{})", pct, total_written, size);
            let _ = std::io::stderr().flush();
            last_progress = total_written;
        }
    }
    eprintln!("\r  Progress: 100.0% ({}/{})", total_written, size);
    eprintln!("Keystream written to: {}", output_path);

    Ok(())
}

// ── SP 800-90B Health Tests (same as entropy_analysis) ────────────────────────

fn compute_shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut counts = [0u64; 256];
    for &b in data {
        counts[b as usize] += 1;
    }
    let total = data.len() as f64;
    let mut entropy = 0.0f64;
    for &count in counts.iter() {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn compute_correlation(data: &[u8]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let n = data.len() - 1;
    let n_f = n as f64;

    let mean_x = data[..n].iter().map(|&b| b as f64).sum::<f64>() / n_f;
    let mean_y = data[1..].iter().map(|&b| b as f64).sum::<f64>() / n_f;

    let mut cov = 0.0f64;
    let mut var_x = 0.0f64;
    let mut var_y = 0.0f64;

    for i in 0..n {
        let xi = data[i] as f64;
        let yi = data[i + 1] as f64;
        let dx = xi - mean_x;
        let dy = yi - mean_y;
        cov += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }

    if var_x == 0.0 || var_y == 0.0 {
        return 0.0;
    }
    cov / (var_x * var_y).sqrt()
}

fn repetition_test(data: &[u8]) -> (bool, usize) {
    if data.len() < 2 {
        return (true, 0);
    }
    let mut max_consecutive = 1usize;
    let mut consecutive = 1usize;
    let mut prev = data[0];

    for &b in &data[1..] {
        if b == prev {
            consecutive += 1;
            if consecutive > max_consecutive {
                max_consecutive = consecutive;
            }
        } else {
            consecutive = 1;
        }
        prev = b;
    }
    (max_consecutive <= 5, max_consecutive)
}

fn adaptive_proportion_test(data: &[u8], window_size: usize) -> (bool, usize, usize) {
    if data.len() < window_size {
        return (true, 0, 0);
    }
    let cutoff = 16;

    let mut counts = [0u32; 256];
    for &b in &data[..window_size] {
        counts[b as usize] += 1;
    }
    let mut worst_count = *counts.iter().max().unwrap_or(&0) as usize;
    let mut worst_offset = 0usize;

    for i in 0..(data.len() - window_size) {
        counts[data[i] as usize] -= 1;
        counts[data[i + window_size] as usize] += 1;
        let max_in_window = *counts.iter().max().unwrap_or(&0) as usize;
        if max_in_window > worst_count {
            worst_count = max_in_window;
            worst_offset = i + 1;
        }
    }
    (worst_count <= cutoff, worst_count, worst_offset)
}

fn runs_test_bit_level(data: &[u8]) -> (bool, u64, u64) {
    if data.len() < 8 {
        return (true, 0, 0);
    }
    let total_bits = (data.len() * 8) as u64;
    let mut runs: u64 = 1;
    let mut prev_bit = (data[0] >> 7) & 1;

    for (i, &byte) in data.iter().enumerate() {
        let start_bit = if i == 0 { 6 } else { 7 };
        for bit_pos in (0..=start_bit).rev() {
            let current_bit = (byte >> bit_pos) & 1;
            if current_bit != prev_bit {
                runs += 1;
                prev_bit = current_bit;
            }
        }
    }

    let expected_runs = total_bits as f64 / 2.0;
    // NIST SP 800-22 §2.3: z = |V_obs - 2nπ(1-π)| / (2·√(2n)·π·(1-π))
    // For π = 0.5: numerator = |V_obs - n/2|, denominator = √(n/2)
    let z = (runs as f64 - expected_runs).abs() / (total_bits as f64 / 2.0).sqrt();
    let ok = z < 2.576;

    (ok, runs, expected_runs as u64)
}

fn longest_run_bit_test(data: &[u8]) -> (bool, u64, u64) {
    if data.is_empty() {
        return (true, 0, 0);
    }
    let total_bits = (data.len() * 8) as u64;

    let max_allowed = if total_bits < 100 {
        12
    } else if total_bits < 1000 {
        16
    } else if total_bits < 10000 {
        20
    } else {
        26
    };

    let mut longest: u64 = 1;
    let mut current: u64 = 1;
    let mut prev_bit = (data[0] >> 7) & 1;

    for (i, &byte) in data.iter().enumerate() {
        let start_bit = if i == 0 { 6 } else { 7 };
        for bit_pos in (0..=start_bit).rev() {
            let current_bit = (byte >> bit_pos) & 1;
            if current_bit == prev_bit {
                current += 1;
                if current > longest {
                    longest = current;
                }
            } else {
                current = 1;
                prev_bit = current_bit;
            }
        }
    }

    (longest <= max_allowed, longest, max_allowed)
}

fn analyze_keystream_file(path: &str) -> Result<(), String> {
    let mut file =
        fs::File::open(path).map_err(|e| format!("Failed to open keystream file: {}", e))?;

    // Read the first 1 MB for analysis (or the whole file if smaller)
    let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let analyze_size = std::cmp::min(file_size, 1024 * 1024) as usize;
    let mut data = vec![0u8; analyze_size];

    file.read_exact(&mut data).map_err(|e| format!("Failed to read keystream: {}", e))?;

    if file_size > 1024 * 1024 {
        eprintln!(
            "  (Note: analyzing first 1 MiB of a {}-byte file — this is a quick health check, not a full NIST analysis)",
            file_size
        );
    }
    eprintln!("Analyzing {} bytes from: {}", analyze_size, path);

    // ── Shannon Entropy ────────────────────────────────────────────────────
    let shannon = compute_shannon_entropy(&data);
    println!("  Shannon Entropy: {:.4} bits/byte (max 8.0)", shannon);
    if shannon > 7.5 {
        println!("  [PASS] Near-maximal entropy (good randomness)");
    } else if shannon > 6.0 {
        println!("  [WARN] Moderate entropy");
    } else {
        println!("  [FAIL] Low entropy");
    }

    // ── Correlation ─────────────────────────────────────────────────────────
    let corr = compute_correlation(&data);
    let corr_abs = corr.abs();
    println!("  Adjacent-byte Correlation: {:.6} (expected ~0)", corr);
    if corr_abs < 0.01 {
        println!("  [PASS] No significant correlation detected");
    } else if corr_abs < 0.05 {
        println!("  [WARN] Weak correlation detected");
    } else {
        println!("  [FAIL] Strong correlation detected");
    }

    // ── Byte distribution ───────────────────────────────────────────────────
    let mut counts = [0u64; 256];
    for &b in &data {
        counts[b as usize] += 1;
    }
    let min_count = *counts.iter().min().unwrap_or(&0);
    let max_count = *counts.iter().max().unwrap_or(&0);
    let expected = data.len() as f64 / 256.0;
    let missing: usize = counts.iter().filter(|&&c| c == 0).count();
    let chi_square: f64 = counts
        .iter()
        .map(|&c| {
            let diff = c as f64 - expected;
            diff * diff / expected
        })
        .sum();
    println!(
        "  Byte distribution: min={}, max={}, expected={:.0}, χ²={:.1}",
        min_count, max_count, expected, chi_square
    );
    if missing > 0 {
        println!("  [FAIL] {} byte values never appear in keystream", missing);
    } else if chi_square < 310.0 {
        println!("  [PASS] Chi-square = {:.1} (critical: 310, df=255)", chi_square);
    } else {
        println!("  [FAIL] Chi-square = {:.1} exceeds critical value 310", chi_square);
    }

    // ── SP 800-90B Health Tests ────────────────────────────────────────────
    println!("\n  ── SP 800-90B Entropy Health Tests ──");

    let (rep_pass, max_cons) = repetition_test(&data);
    println!(
        "  [{}] Repetition Test (max {} consecutive identical bytes)",
        if rep_pass { "PASS" } else { "FAIL" },
        max_cons
    );

    let (apt_pass, worst_count, worst_off) = adaptive_proportion_test(&data, 512);
    println!(
        "  [{}] Adaptive Proportion Test (worst window: {}/512 at offset {})",
        if apt_pass { "PASS" } else { "FAIL" },
        worst_count,
        worst_off
    );

    let (runs_pass, runs, expected_runs) = runs_test_bit_level(&data);
    println!(
        "  [{}] Runs Test ({} runs, expected ~{})",
        if runs_pass { "PASS" } else { "FAIL" },
        runs,
        expected_runs
    );

    let (longest_pass, longest, max_allowed) = longest_run_bit_test(&data);
    println!(
        "  [{}] Longest Run Test (longest: {} bits, max allowed: {})",
        if longest_pass { "PASS" } else { "FAIL" },
        longest,
        max_allowed
    );

    // ── Summary ────────────────────────────────────────────────────────────
    let results = [
        ("Shannon Entropy", shannon > 7.5),
        ("Correlation", corr_abs < 0.01),
        ("Byte Distribution", missing == 0 && chi_square < 310.0),
        ("Repetition Test", rep_pass),
        ("Adaptive Proportion", apt_pass),
        ("Runs Test", runs_pass),
        ("Longest Run Test", longest_pass),
    ];

    let passed = results.iter().filter(|(_, ok)| *ok).count();
    let total = results.len();

    println!("\n  ── Summary ──");
    for (name, ok) in &results {
        println!(
            "  {} {} [{}]",
            if *ok { "✓" } else { "✗" },
            name,
            if *ok { "PASS" } else { "FAIL" }
        );
    }
    println!(
        "  Result: {}/{} tests passed {}",
        passed,
        total,
        if passed == total { "✅" } else { "⚠️" }
    );

    Ok(())
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    match args.command {
        Command::Generate { size, output, config, verlet } => {
            let method = if verlet { IntegrationMethod::Verlet } else { IntegrationMethod::Euler };

            let orbital_config = if let Some(config_path) = &config {
                let json = fs::read_to_string(config_path).unwrap_or_else(|e| {
                    eprintln!("Failed to read config file '{}': {}", config_path, e);
                    std::process::exit(1);
                });
                OrbitalConfig::from_json(&json).unwrap_or_else(|e| {
                    eprintln!("Failed to parse config: {:?}", e);
                    std::process::exit(1);
                })
            } else {
                default_config()
            };

            let method_label = if verlet { "Verlet" } else { "Euler" };
            eprintln!("NIST SP 800-90B Keystream Generator");
            eprintln!("  Size: {} bytes ({:.2} GB)", size, size as f64 / 1_073_741_824.0);
            eprintln!("  Output: {}", output);
            eprintln!("  Integration: {}", method_label);
            eprintln!("  Config: {}", if config.is_some() { "custom" } else { "default 5-body" });

            if let Err(e) = generate_keystream(orbital_config, size, method, &output) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
        Command::Analyze { input } => {
            if !Path::new(&input).exists() {
                eprintln!("Error: file '{}' not found", input);
                std::process::exit(1);
            }
            if let Err(e) = analyze_keystream_file(&input) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        },
    }
}
