//! NIST SP 800-22 Statistical Test Suite for Kelvin Orbital Keystream
//!
//! Uses the `nistrs` crate (https://crates.io/crates/nistrs) for all 15 NIST tests.
//! Generates a keystream from a Kelvin orbital configuration and validates
//! randomness quality against the NIST SP 800-22 Rev 1a standard.
//!
//! Tests:
//!   1. Frequency (Monobit) Test
//!   2. Frequency Test within a Block
//!   3. Runs Test
//!   4. Longest Run of Ones in a Block
//!   5. Binary Matrix Rank Test
//!   6. Discrete Fourier Transform (Spectral) Test
//!   7. Non-overlapping Template Matching
//!   8. Overlapping Template Matching
//!   9. Maurer's Universal Statistical Test
//!   10. Linear Complexity Test
//!   11. Serial Test
//!   12. Approximate Entropy Test
//!   13. Cumulative Sums (Cusum) Test
//!   14. Random Excursions Test
//!   15. Random Excursions Variant Test

use std::fs::File;
use std::io::Write;

use nistrs::prelude::*;
use nistrs::BitsData;

fn main() {
    println!("=== NIST SP 800-22 Statistical Test Suite for Kelvin ===\n");

    // Generate a keystream from a Kelvin orbital configuration
    let keystream = generate_keystream();
    let data = BitsData::from_binary(keystream.clone());
    let n = data.len();

    println!("Keystream size: {} bytes = {} bits", keystream.len(), n);
    println!();

    // Run all 15 NIST tests
    let mut passed = 0u32;
    let mut total = 0u32;

    // 1. Frequency (Monobit) Test
    total += 1;
    let (ok, p) = frequency_test(&data);
    print_result("Frequency (Monobit) Test", ok, p);
    if ok { passed += 1; }

    // 2. Frequency Test within a Block
    total += 1;
    match block_frequency_test(&data, 128) {
        Ok((ok, p)) => {
            print_result("Block Frequency Test (M=128)", ok, p);
            if ok { passed += 1; }
        }
        Err(e) => println!("  [FAIL] Block Frequency Test: {}", e),
    }

    // 3. Runs Test
    total += 1;
    let (ok, p) = runs_test(&data);
    print_result("Runs Test", ok, p);
    if ok { passed += 1; }

    // 4. Longest Run of Ones in a Block
    total += 1;
    match longest_run_of_ones_test(&data) {
        Ok((ok, p)) => {
            print_result("Longest Run of Ones Test", ok, p);
            if ok { passed += 1; }
        }
        Err(e) => println!("  [FAIL] Longest Run of Ones Test: {}", e),
    }

    // 5. Binary Matrix Rank Test
    total += 1;
    match rank_test(&data) {
        Ok((ok, p)) => {
            print_result("Binary Matrix Rank Test", ok, p);
            if ok { passed += 1; }
        }
        Err(e) => println!("  [FAIL] Binary Matrix Rank Test: {}", e),
    }

    // 6. Discrete Fourier Transform (Spectral) Test
    total += 1;
    let (ok, p) = fft_test(&data);
    print_result("Discrete Fourier Transform Test", ok, p);
    if ok { passed += 1; }

    // 7. Non-overlapping Template Matching
    total += 1;
    match non_overlapping_template_test(&data, 9) {
        Ok(results) => {
            // NIST STS approach: test passes if >= 97.5% of templates pass
            let pass_count = results.iter().filter(|(ok, _)| *ok).count();
            let pass_rate = pass_count as f64 / results.len() as f64;
            let ok = pass_rate >= 0.975;
            println!("  [{}] Non-overlapping Template Matching Test (m=9, {} templates, {:.1}% pass rate)",
                if ok { "PASS" } else { "FAIL" },
                results.len(),
                pass_rate * 100.0);
            if ok { passed += 1; }
        }
        Err(e) => {
            println!("  [FAIL] Non-overlapping Template Matching Test: {}", e);
        }
    }

    // 8. Overlapping Template Matching
    total += 1;
    let (ok, p) = overlapping_template_test(&data, 9);
    print_result("Overlapping Template Matching Test", ok, p);
    if ok { passed += 1; }

    // 9. Maurer's Universal Statistical Test
    total += 1;
    let (ok, p) = universal_test(&data);
    print_result("Maurer's Universal Statistical Test", ok, p);
    if ok { passed += 1; }

    // 10. Linear Complexity Test
    total += 1;
    let (ok, p) = linear_complexity_test(&data, 500);
    print_result("Linear Complexity Test", ok, p);
    if ok { passed += 1; }

    // 11. Serial Test
    total += 1;
    let results = serial_test(&data, 16);
    let ok = results[0].0 && results[1].0;
    println!("  [{}] Serial Test (m=16) — del1_p={:.6}, del2_p={:.6}",
        if ok { "PASS" } else { "FAIL" },
        results[0].1, results[1].1);
    if ok { passed += 1; }

    // 12. Approximate Entropy Test
    total += 1;
    let (ok, p) = approximate_entropy_test(&data, 2);
    print_result("Approximate Entropy Test (m=2)", ok, p);
    if ok { passed += 1; }

    // 13. Cumulative Sums (Cusum) Test
    total += 1;
    let results = cumulative_sums_test(&data);
    let ok = results[0].0 && results[1].0;
    println!("  [{}] Cumulative Sums Test — forward_p={:.6}, reverse_p={:.6}",
        if ok { "PASS" } else { "FAIL" },
        results[0].1, results[1].1);
    if ok { passed += 1; }

    // 14. Random Excursions Test
    total += 1;
    match random_excursions_test(&data) {
        Ok(results) => {
            // All 8 states must pass
            let ok = results.iter().all(|(ok, _)| *ok);
            let min_p = results.iter().map(|(_, p)| *p).fold(1.0f64, f64::min);
            println!("  [{}] Random Excursions Test — min_p={:.6}",
                if ok { "PASS" } else { "FAIL" }, min_p);
            if ok { passed += 1; }
        }
        Err(e) => {
            println!("  [FAIL] Random Excursions Test: {}", e);
        }
    }

    // 15. Random Excursions Variant Test
    total += 1;
    match random_excursions_variant_test(&data) {
        Ok(results) => {
            // All 18 states must pass
            let ok = results.iter().all(|(ok, _)| *ok);
            let min_p = results.iter().map(|(_, p)| *p).fold(1.0f64, f64::min);
            println!("  [{}] Random Excursions Variant Test — min_p={:.6}",
                if ok { "PASS" } else { "FAIL" }, min_p);
            if ok { passed += 1; }
        }
        Err(e) => {
            println!("  [FAIL] Random Excursions Variant Test: {}", e);
        }
    }

    println!();
    println!("=== Results: {}/{} tests passed ===", passed, total);

    if passed == total {
        println!("✅ Kelvin orbital keystream passes all NIST SP 800-22 tests.");
    } else {
        println!("⚠️  Some tests failed. Review configuration.");
    }

    // Export keystream for external validation
    let mut f = File::create("keystream.bin").expect("Failed to create keystream.bin");
    f.write_all(&keystream).expect("Failed to write keystream");
    println!("Keystream exported to keystream.bin for external validation.");
}

/// Print a single test result with its p-value.
fn print_result(name: &str, ok: bool, p: f64) {
    println!("  [{}] {} — p={:.6}",
        if ok { "PASS" } else { "FAIL" }, name, p);
}

/// Generate a keystream from a Kelvin orbital configuration
fn generate_keystream() -> Vec<u8> {
    use kelvin::OrbitalConfig;
    use kelvin::Kelvin;
    use kelvin_core::{Fixed, Vec3, OrbitalBody, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

    // Build a deterministic test configuration with distinct positions
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
    let bodies = vec![sun, planet1, planet2, planet3, planet4];
    let config = OrbitalConfig::new(
        bodies,
        1000,          // total_steps
        10,            // reseed_interval
        DEFAULT_DT,    // dt
        SOFTENING_FACTOR, // softening
        DEFAULT_G,     // G
    ).expect("Failed to create config");
    let mut kelvin = Kelvin::new(config).expect("Failed to initialize Kelvin");

    // Generate 1MB of keystream by encrypting zeros
    let mut plaintext = vec![0u8; 1_048_576]; // 1 MB
    kelvin.encrypt(&mut plaintext).expect("Failed to encrypt");
    plaintext
}
