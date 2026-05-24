//! NIST SP 800-22 Statistical Test Suite for Kelvin Cryptosystem
//!
//! Tests all 4 variants across all recommended parameters:
//!   - V1 Secure (Kelvin): ChaCha20Poly1305 AEAD, Verlet + Euler
//!   - V2 Chaos (KelvinStreaming): Per-step SHAKE256 XOR, Verlet + Euler
//!   - V3 Photon (KelvinPhoton): HKDF→SHAKE256 bulk XOR
//!   - H Quantum (KelvinQuantum): Hybrid V3 cache + V2 orbital reseed
//!
//! Uses the `nistrs` crate (https://crates.io/crates/nistrs) for all 15 NIST tests.
//! Generates 1 MB keystream per variant by encrypting zeros.

use std::fs::File;
use std::io::Write;

use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};
use nistrs::prelude::*;
use nistrs::BitsData;

fn main() {
    println!("=== NIST SP 800-22 Statistical Test Suite for Kelvin ===\n");

    // Generate the base 2048-byte seed from orbital simulation
    let (seed, _bodies) = generate_base_seed();

    let mut all_passed = Vec::new();
    let mut all_total = Vec::new();

    // ── V1 Secure: ChaCha20Poly1305 AEAD ──────────────────────────────────
    println!("--- V1 Secure (ChaCha20Poly1305 / Verlet) ---");
    let v1_verlet_keystream = generate_v1_keystream_verlet();
    let data = BitsData::from_binary(v1_verlet_keystream.clone());
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    println!();
    println!("--- V1 Secure (ChaCha20Poly1305 / Euler) ---");
    let keystream = generate_v1_keystream_euler();
    let data = BitsData::from_binary(keystream);
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    // ── V2 Chaos: Per-step SHAKE256 XOR ───────────────────────────────────
    println!();
    println!("--- V2 Chaos (Stream XOR / Verlet) ---");
    let keystream = generate_v2_keystream_verlet();
    let data = BitsData::from_binary(keystream);
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    println!();
    println!("--- V2 Chaos (Stream XOR / Euler) ---");
    let keystream = generate_v2_keystream_euler();
    let data = BitsData::from_binary(keystream);
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    // ── V3 Photon: HKDF→SHAKE256 bulk XOR ─────────────────────────────────
    println!();
    println!("--- V3 Photon (HKDF→SHAKE256) ---");
    let keystream = generate_v3_keystream(seed);
    let data = BitsData::from_binary(keystream);
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    // ── H Quantum: Hybrid V3 cache + V2 orbital reseed ────────────────────
    println!();
    println!("--- H Quantum (Hybrid) ---");
    let keystream = generate_h_keystream(seed);
    let data = BitsData::from_binary(keystream);
    let (p, t) = run_all_tests(&data);
    all_passed.push(p);
    all_total.push(t);

    // ── Summary ────────────────────────────────────────────────────────────
    println!();
    println!("=== Summary ===");
    let labels = ["V1 Verlet", "V1 Euler", "V2 Verlet", "V2 Euler", "V3 Photon", "H Quantum"];
    for (i, label) in labels.iter().enumerate() {
        let status = if all_passed[i] == all_total[i] { "✅" } else { "⚠️" };
        println!("  {}: {}/{} {}", label, all_passed[i], all_total[i], status);
    }

    // Export keystream for external validation (reuse V1 Verlet bytes from above)
    let mut f = File::create("keystream.bin").expect("Failed to create keystream.bin");
    f.write_all(&v1_verlet_keystream).expect("Failed to write keystream");
    println!();
    println!("Keystream exported to keystream.bin for external validation.");
}

/// Run all 15 NIST SP 800-22 tests on the given bit data.
/// Returns (passed, total).
fn run_all_tests(data: &BitsData) -> (u32, u32) {
    let n = data.len();
    println!("  Keystream: {} bytes = {} bits", n / 8, n);
    println!();

    let mut passed = 0u32;
    let mut total = 0u32;

    // 1. Frequency (Monobit) Test
    total += 1;
    let (ok, p) = frequency_test(data);
    print_result("Frequency (Monobit) Test", ok, p);
    if ok {
        passed += 1;
    }

    // 2. Frequency Test within a Block
    total += 1;
    match block_frequency_test(data, 128) {
        Ok((ok, p)) => {
            print_result("Block Frequency Test (M=128)", ok, p);
            if ok {
                passed += 1;
            }
        },
        Err(e) => println!("  [FAIL] Block Frequency Test: {}", e),
    }

    // 3. Runs Test
    total += 1;
    let (ok, p) = runs_test(data);
    print_result("Runs Test", ok, p);
    if ok {
        passed += 1;
    }

    // 4. Longest Run of Ones in a Block
    total += 1;
    match longest_run_of_ones_test(data) {
        Ok((ok, p)) => {
            print_result("Longest Run of Ones Test", ok, p);
            if ok {
                passed += 1;
            }
        },
        Err(e) => println!("  [FAIL] Longest Run of Ones Test: {}", e),
    }

    // 5. Binary Matrix Rank Test
    total += 1;
    match rank_test(data) {
        Ok((ok, p)) => {
            print_result("Binary Matrix Rank Test", ok, p);
            if ok {
                passed += 1;
            }
        },
        Err(e) => println!("  [FAIL] Binary Matrix Rank Test: {}", e),
    }

    // 6. Discrete Fourier Transform (Spectral) Test
    total += 1;
    let (ok, p) = fft_test(data);
    print_result("Discrete Fourier Transform Test", ok, p);
    if ok {
        passed += 1;
    }

    // 7. Non-overlapping Template Matching
    total += 1;
    match non_overlapping_template_test(data, 9) {
        Ok(results) => {
            let pass_count = results.iter().filter(|(ok, _)| *ok).count();
            let pass_rate = pass_count as f64 / results.len() as f64;
            let ok = pass_rate >= 0.975;
            println!("  [{}] Non-overlapping Template Matching Test (m=9, {} templates, {:.1}% pass rate)",
                if ok { "PASS" } else { "FAIL" },
                results.len(),
                pass_rate * 100.0);
            if ok {
                passed += 1;
            }
        },
        Err(e) => {
            println!("  [FAIL] Non-overlapping Template Matching Test: {}", e);
        },
    }

    // 8. Overlapping Template Matching
    total += 1;
    let (ok, p) = overlapping_template_test(data, 9);
    print_result("Overlapping Template Matching Test", ok, p);
    if ok {
        passed += 1;
    }

    // 9. Maurer's Universal Statistical Test
    total += 1;
    let (ok, p) = universal_test(data);
    print_result("Maurer's Universal Statistical Test", ok, p);
    if ok {
        passed += 1;
    }

    // 10. Linear Complexity Test
    total += 1;
    let (ok, p) = linear_complexity_test(data, 500);
    print_result("Linear Complexity Test", ok, p);
    if ok {
        passed += 1;
    }

    // 11. Serial Test
    total += 1;
    let results = serial_test(data, 16);
    let ok = results.len() >= 2 && results[0].0 && results[1].0;
    let del1_p = if !results.is_empty() { results[0].1 } else { 0.0 };
    let del2_p = if results.len() >= 2 { results[1].1 } else { 0.0 };
    println!(
        "  [{}] Serial Test (m=16) — del1_p={:.6}, del2_p={:.6}",
        if ok { "PASS" } else { "FAIL" },
        del1_p,
        del2_p
    );
    if ok {
        passed += 1;
    }

    // 12. Approximate Entropy Test
    total += 1;
    let (ok, p) = approximate_entropy_test(data, 2);
    print_result("Approximate Entropy Test (m=2)", ok, p);
    if ok {
        passed += 1;
    }

    // 13. Cumulative Sums (Cusum) Test
    total += 1;
    let results = cumulative_sums_test(data);
    let ok = results.len() >= 2 && results[0].0 && results[1].0;
    let forward_p = if !results.is_empty() { results[0].1 } else { 0.0 };
    let reverse_p = if results.len() >= 2 { results[1].1 } else { 0.0 };
    println!(
        "  [{}] Cumulative Sums Test — forward_p={:.6}, reverse_p={:.6}",
        if ok { "PASS" } else { "FAIL" },
        forward_p,
        reverse_p
    );
    if ok {
        passed += 1;
    }

    // 14. Random Excursions Test
    total += 1;
    match random_excursions_test(data) {
        Ok(results) => {
            let ok = results.iter().all(|(ok, _)| *ok);
            let min_p = results.iter().map(|(_, p)| *p).fold(1.0f64, f64::min);
            println!(
                "  [{}] Random Excursions Test — min_p={:.6}",
                if ok { "PASS" } else { "FAIL" },
                min_p
            );
            if ok {
                passed += 1;
            }
        },
        Err(e) => {
            println!("  [FAIL] Random Excursions Test: {}", e);
        },
    }

    // 15. Random Excursions Variant Test
    total += 1;
    match random_excursions_variant_test(data) {
        Ok(results) => {
            let ok = results.iter().all(|(ok, _)| *ok);
            let min_p = results.iter().map(|(_, p)| *p).fold(1.0f64, f64::min);
            println!(
                "  [{}] Random Excursions Variant Test — min_p={:.6}",
                if ok { "PASS" } else { "FAIL" },
                min_p
            );
            if ok {
                passed += 1;
            }
        },
        Err(e) => {
            println!("  [FAIL] Random Excursions Variant Test: {}", e);
        },
    }

    println!();
    println!("  Results: {}/{} tests passed", passed, total);
    if passed == total {
        println!("  ✅ All tests passed.");
    } else {
        println!("  ⚠️  Some tests failed.");
    }

    (passed, total)
}

/// Print a single test result with its p-value.
fn print_result(name: &str, ok: bool, p: f64) {
    println!("  [{}] {} — p={:.6}", if ok { "PASS" } else { "FAIL" }, name, p);
}

// ── Base seed generation ─────────────────────────────────────────────────────

/// Generate the base 2048-byte seed from a 5-body orbital simulation.
fn generate_base_seed() -> ([u8; 2048], Vec<OrbitalBody>) {
    use kelvin::OrbitalConfig;

    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 1000, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    kelvin::simulate_and_extract_seed(&config).expect("Failed to extract seed")
}

fn default_bodies() -> Vec<OrbitalBody> {
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
    vec![sun, planet1, planet2, planet3, planet4]
}

// ── V1 Secure: ChaCha20Poly1305 AEAD ─────────────────────────────────────────

fn generate_v1_keystream_verlet() -> Vec<u8> {
    use kelvin::{IntegrationMethod, Kelvin, OrbitalConfig};

    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 2000, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let mut kelvin = Kelvin::new_with_method(config, IntegrationMethod::Verlet)
        .expect("Failed to initialize Kelvin");

    let mut plaintext = vec![0u8; 1_048_576 + 16]; // 1 MB + AEAD tag space
    kelvin.encrypt(&mut plaintext).expect("Failed to encrypt");
    plaintext[..1_048_576].to_vec()
}

fn generate_v1_keystream_euler() -> Vec<u8> {
    use kelvin::{IntegrationMethod, Kelvin, OrbitalConfig};

    // Use a more stable Euler configuration: fewer steps, smaller dt
    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 500, 5, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let mut kelvin = Kelvin::new_with_method(config, IntegrationMethod::Euler)
        .expect("Failed to initialize Kelvin");

    let mut plaintext = vec![0u8; 1_048_576 + 16]; // 1 MB + AEAD tag space
    kelvin.encrypt(&mut plaintext).expect("Failed to encrypt");
    plaintext[..1_048_576].to_vec()
}

// ── V2 Chaos: Per-step SHAKE256 XOR ──────────────────────────────────────────

fn generate_v2_keystream_verlet() -> Vec<u8> {
    use kelvin::{KelvinStreaming, OrbitalConfig};

    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 2000, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let mut ks =
        KelvinStreaming::new(config, 1_048_576).expect("Failed to initialize KelvinStreaming");

    let mut data = vec![0u8; 1_048_576];
    ks.encrypt(&mut data).expect("Failed to encrypt");
    data
}

fn generate_v2_keystream_euler() -> Vec<u8> {
    use kelvin::{IntegrationMethod, KelvinStreaming, OrbitalConfig};

    // Use a more stable Euler configuration: fewer steps, smaller dt
    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 500, 5, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let mut ks = KelvinStreaming::new_with_method(config, 1_048_576, IntegrationMethod::Euler)
        .expect("Failed to initialize KelvinStreaming");

    let mut data = vec![0u8; 1_048_576];
    ks.encrypt(&mut data).expect("Failed to encrypt");
    data
}

// ── V3 Photon: HKDF→SHAKE256 bulk XOR ────────────────────────────────────────

fn generate_v3_keystream(seed: [u8; 2048]) -> Vec<u8> {
    use kelvin::KelvinPhoton;

    let mut photon = KelvinPhoton::new(seed, 1000);
    let mut data = vec![0u8; 1_048_576];
    photon.encrypt(&mut data).expect("Failed to encrypt");
    data
}

// ── H Quantum: Hybrid V3 cache + V2 orbital reseed ───────────────────────────

fn generate_h_keystream(seed: [u8; 2048]) -> Vec<u8> {
    use kelvin::KelvinQuantum;

    let mut quantum = KelvinQuantum::new(seed, 1000);
    let mut data = vec![0u8; 1_048_576];
    quantum.encrypt(&mut data).expect("Failed to encrypt");
    data
}
