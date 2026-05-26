#![allow(clippy::disallowed_methods)]

//! Entropy Analysis and SP 800-90B Health Tests for Kelvin Cryptosystem
//!
//! Two modes:
//!   --keystream   Generate keystream and run statistical tests
//!   --keys N      Generate N keys and analyze orbital parameter entropy
//!
//! SP 800-90B tests ported from latticearc's entropy_tests.rs:
//!   - Repetition Test (§4.4.1)
//!   - Adaptive Proportion Test (§4.4.2)
//!   - Runs Test (§2.3 simplified)
//!   - Longest Run Test (§2.4 simplified)

use std::fs;
use std::path::Path;

use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};

// ── CLI argument parsing (no clap dependency) ─────────────────────────────────

struct Args {
    keystream: bool,
    num_keys: usize,
    level: String,
    dir: String,
    output: Option<String>,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut keystream = false;
    let mut num_keys = 100usize;
    let mut level = "standard".to_string();
    let mut dir = "test_results".to_string();
    let mut output: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--keystream" => keystream = true,
            "--keys" => {
                i += 1;
                if i < args.len() {
                    num_keys = args[i].parse().unwrap_or(100);
                }
            },
            "--level" => {
                i += 1;
                if i < args.len() {
                    level = args[i].clone();
                }
            },
            "--dir" => {
                i += 1;
                if i < args.len() {
                    dir = args[i].clone();
                }
            },
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output = Some(args[i].clone());
                }
            },
            _ => {},
        }
        i += 1;
    }
    Args { keystream, num_keys, level, dir, output }
}

// ── Keystream generation ──────────────────────────────────────────────────────

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

fn generate_keystream() -> Vec<u8> {
    use kelvin::{IntegrationMethod, Kelvin, OrbitalConfig};

    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 2000, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let mut kelvin = Kelvin::new_with_method(config, IntegrationMethod::default())
        .expect("Failed to init Kelvin");

    let mut plaintext = vec![0u8; 1_048_576 + 16]; // 1 MB + AEAD tag
    kelvin.encrypt(&mut plaintext).expect("Failed to encrypt");
    plaintext[..1_048_576].to_vec()
}

fn generate_prism_keystream() -> Vec<u8> {
    use kelvin::{simulate_and_extract_seed, KelvinPrism, OrbitalConfig};

    let bodies = default_bodies();
    let config = OrbitalConfig::new(bodies, 1000, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("Failed to create config");

    let (seed, _bodies) = simulate_and_extract_seed(&config).expect("Failed to extract seed");

    let mut prism = KelvinPrism::new(seed, 1000);
    let mut data = vec![0u8; 1_048_576];
    prism.encrypt(&mut data).expect("Failed to encrypt");
    data
}

// ── Shannon Entropy ───────────────────────────────────────────────────────────

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

// ── Adjacent-byte Correlation (Pearson) ───────────────────────────────────────

fn compute_correlation(data: &[u8]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let n = data.len() - 1;
    let n_f = n as f64;

    // Single pass: compute means first
    let mean_x = data[..n].iter().map(|&b| b as f64).sum::<f64>() / n_f;
    let mean_y = data[1..].iter().map(|&b| b as f64).sum::<f64>() / n_f;

    // Second pass: compute covariance and variances
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

// ── SP 800-90B §4.4.1 Repetition Test ─────────────────────────────────────────

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

// ── SP 800-90B §4.4.2 Adaptive Proportion Test ────────────────────────────────

fn adaptive_proportion_test(data: &[u8], window_size: usize) -> (bool, usize, usize) {
    if data.len() < window_size {
        return (true, 0, 0);
    }
    let cutoff = 16; // CRITBINOM(512, 2^-8, 1-2^-30) + 1

    // Initialize sliding window counts
    let mut counts = [0u32; 256];
    for &b in &data[..window_size] {
        counts[b as usize] += 1;
    }
    let mut worst_count = *counts.iter().max().unwrap_or(&0) as usize;
    let mut worst_offset = 0usize;

    // Slide the window: O(N) instead of O(N*W)
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

// ── SP 800-22 §2.3 Runs Test (simplified) ─────────────────────────────────────

fn runs_test_bit_level(data: &[u8]) -> (bool, u64, u64) {
    if data.len() < 8 {
        return (true, 0, 0);
    }
    let total_bits = (data.len() * 8) as u64;
    let mut runs: u64 = 1;
    let mut prev_bit = (data[0] >> 7) & 1;

    for &byte in data.iter() {
        for bit_pos in (0..=7).rev() {
            let current_bit = (byte >> bit_pos) & 1;
            if current_bit != prev_bit {
                runs += 1;
                prev_bit = current_bit;
            }
        }
    }

    let expected_runs = total_bits as f64 / 2.0;
    // NIST SP 800-22 §2.3 z-statistic:
    //   Expected runs: n/2
    //   Variance: n/4 → std. dev.: √n/2
    //   z = |runs - n/2| / (√n/2) = 2|runs - n/2| / √n
    // Reject if |z| >= 2.576 (significance level α = 0.01)
    let z = 2.0 * (runs as f64 - expected_runs).abs() / (total_bits as f64).sqrt();
    let ok = z < 2.576;

    (ok, runs, expected_runs as u64)
}

// ── SP 800-22 §2.4 Longest Run Test (block-based) ─────────────────────────────
//
// For large datasets (≥ 1,000,000 bits), NIST SP 800-22 §2.4 uses a
// chi-square test across blocks of size M = 10000. The critical values
// for the longest run within each block are:
//   ≤ 10, 11, 12, 13, 14, 15, 16, ≥ 17
// with expected frequencies v = [0.0882, 0.2092, 0.2483, 0.1933,
//                                0.1208, 0.0675, 0.0727] × N_blocks.
//
// For smaller datasets, we use the simplified global threshold approach.

/// Result of the longest run test.
struct LongestRunResult {
    pass: bool,
    /// The observed longest run (in bits) across the entire dataset.
    longest_run: u64,
    /// Human-readable description of the threshold/criterion used.
    detail: String,
}

fn longest_run_bit_test(data: &[u8]) -> LongestRunResult {
    if data.is_empty() {
        return LongestRunResult { pass: true, longest_run: 0, detail: "no data".to_string() };
    }
    let total_bits = (data.len() * 8) as u64;

    // For datasets ≥ 1,000,000 bits, use NIST SP 800-22 §2.4 block-based test
    if total_bits >= 1_000_000 {
        const M: u64 = 10000; // block size
        let n_blocks = (total_bits / M) as usize;

        // NIST SP 800-22 §2.4 Table 2.7 (M = 10000, 7 categories):
        //   v[0]=10 (≤10), v[1]=11, v[2]=12, v[3]=13, v[4]=14, v[5]=15, v[6]=16 (≥16)
        //   pi = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727]
        let expected_probs = [0.0882, 0.2092, 0.2483, 0.1933, 0.1208, 0.0675, 0.0727];
        let expected: Vec<f64> = expected_probs.iter().map(|p| p * n_blocks as f64).collect();

        // Count longest run in each block (7 categories matching NIST SP 800-22)
        let mut counts = [0u64; 7]; // indices: 0=≤10, 1=11, 2=12, 3=13, 4=14, 5=15, 6=≥16
        let mut global_longest: u64 = 1;

        for block in 0..n_blocks {
            let start_bit = block as u64 * M;

            // NIST SP 800-22 §2.4: count longest run of ONES only
            let mut longest: u64 = 0;
            let mut current: u64 = 0;

            for bit_idx in 0..M {
                let abs_bit = start_bit + bit_idx;
                let byte_idx = (abs_bit / 8) as usize;
                let bit_pos = 7 - (abs_bit % 8) as u8;
                let current_bit = (data[byte_idx] >> bit_pos) & 1;

                if current_bit == 1 {
                    current += 1;
                    if current > longest {
                        longest = current;
                    }
                } else {
                    current = 0;
                }
            }

            if longest > global_longest {
                global_longest = longest;
            }

            // NIST SP 800-22 §2.4: 7 categories matching Table 2.7
            // v[0]=10 (≤10), v[1]=11, v[2]=12, v[3]=13, v[4]=14, v[5]=15, v[6]=16 (≥16)
            let idx = match longest {
                0..=10 => 0,
                11 => 1,
                12 => 2,
                13 => 3,
                14 => 4,
                15 => 5,
                _ => 6, // ≥16
            };
            counts[idx] += 1;
        }

        // Chi-square statistic
        let chi_sq: f64 = counts
            .iter()
            .zip(expected.iter())
            .map(|(&obs, &exp)| {
                if exp > 0.0 {
                    let diff = obs as f64 - exp;
                    diff * diff / exp
                } else {
                    0.0
                }
            })
            .sum();

        // Critical value for df=7 at α=0.01 is 18.475
        let pass = chi_sq < 18.475;
        let detail = format!(
            "χ²={:.2} (critical: 18.475, df=6), longest block run: {} bits, dist: [{}, {}, {}, {}, {}, {}, {}]",
            chi_sq, global_longest,
            counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6]
        );
        return LongestRunResult { pass, longest_run: global_longest, detail };
    }

    // For smaller datasets, use simplified global threshold
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

    for &byte in data.iter() {
        for bit_pos in (0..=7).rev() {
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

    let pass = longest <= max_allowed;
    let detail = format!("longest: {} bits, max allowed: {}", longest, max_allowed);
    LongestRunResult { pass, longest_run: longest, detail }
}

// ── Keystream analysis (returns report string) ────────────────────────────────

fn analyze_keystream() -> String {
    let mut out = String::new();

    out.push_str(&format!("\n{:=^60}\n", ""));
    out.push_str(" KEYSTREAM STATISTICAL ANALYSIS\n");
    out.push_str(&format!("{:=^60}\n", ""));

    out.push_str("\nGenerating 1MB keystream via Kelvin (V1 Verlet)...\n");
    let keystream = generate_keystream();
    out.push_str(&format!("  Keystream size: {} bytes\n", keystream.len()));

    // ── Shannon Entropy ────────────────────────────────────────────────────
    let shannon = compute_shannon_entropy(&keystream);
    out.push_str(&format!("  Shannon Entropy: {:.4} bits/byte (max 8.0)\n", shannon));
    if shannon > 7.5 {
        out.push_str("  [PASS] Near-maximal entropy (good randomness)\n");
    } else if shannon > 6.0 {
        out.push_str("  [WARN] Moderate entropy\n");
    } else {
        out.push_str("  [FAIL] Low entropy\n");
    }

    // ── Correlation Coefficient ─────────────────────────────────────────────
    let corr = compute_correlation(&keystream);
    out.push_str(&format!("  Adjacent-byte Correlation: {:.6} (expected ~0)\n", corr));
    if corr.abs() < 0.01 {
        out.push_str("  [PASS] No significant correlation detected\n");
    } else if corr.abs() < 0.05 {
        out.push_str("  [WARN] Weak correlation detected\n");
    } else {
        out.push_str("  [FAIL] Strong correlation detected\n");
    }

    // ── Byte value distribution (chi-square test) ──────────────────────────
    let mut counts = [0u64; 256];
    for &b in &keystream {
        counts[b as usize] += 1;
    }
    let min_count = *counts.iter().min().unwrap_or(&0);
    let max_count = *counts.iter().max().unwrap_or(&0);
    let expected = keystream.len() as f64 / 256.0;
    let missing: usize = counts.iter().filter(|&&c| c == 0).count();
    let chi_square: f64 = counts
        .iter()
        .map(|&c| {
            let diff = c as f64 - expected;
            diff * diff / expected
        })
        .sum();
    out.push_str(&format!(
        "  Byte value distribution: min={}, max={}, expected={:.0}, χ²={:.1}\n",
        min_count, max_count, expected, chi_square
    ));
    if missing > 0 {
        out.push_str(&format!("  [FAIL] {} byte values never appear in keystream\n", missing));
    } else if chi_square < 310.0 {
        out.push_str(&format!("  [PASS] Chi-square = {:.1} (critical: 310, df=255)\n", chi_square));
    } else {
        out.push_str(&format!(
            "  [FAIL] Chi-square = {:.1} exceeds critical value 310\n",
            chi_square
        ));
    }

    // ── SP 800-90B Entropy Health Tests ────────────────────────────────────
    out.push_str("\n  ── SP 800-90B Entropy Health Tests ──\n");

    // Repetition Test (§4.4.1)
    let (rep_pass, max_cons) = repetition_test(&keystream);
    out.push_str(&format!(
        "  [{}] Repetition Test (max {} consecutive identical bytes)\n",
        if rep_pass { "PASS" } else { "FAIL" },
        max_cons
    ));

    // Adaptive Proportion Test (§4.4.2)
    let (apt_pass, worst_count, worst_off) = adaptive_proportion_test(&keystream, 512);
    out.push_str(&format!(
        "  [{}] Adaptive Proportion Test (worst window: {}/512 at offset {})\n",
        if apt_pass { "PASS" } else { "FAIL" },
        worst_count,
        worst_off
    ));

    // Runs Test (§2.3 simplified)
    let (runs_pass, runs, expected_runs) = runs_test_bit_level(&keystream);
    out.push_str(&format!(
        "  [{}] Runs Test ({} runs, expected ~{})\n",
        if runs_pass { "PASS" } else { "FAIL" },
        runs,
        expected_runs
    ));

    // Longest Run Test (§2.4 simplified)
    let longest_result = longest_run_bit_test(&keystream);
    out.push_str(&format!(
        "  [{}] Longest Run Test ({})\n",
        if longest_result.pass { "PASS" } else { "FAIL" },
        longest_result.detail
    ));

    // ── Summary ────────────────────────────────────────────────────────────
    // Reuse variables already computed above — no re-running expensive tests
    let shannon_pass = shannon > 7.5;
    let corr_pass = corr.abs() < 0.01;
    let dist_pass = missing == 0 && chi_square < 310.0;

    let results = [
        ("Shannon Entropy", shannon_pass, format!("{:.4} bits/byte", shannon)),
        ("Correlation", corr_pass, format!("{:.6}", corr)),
        ("Byte Distribution", dist_pass, format!("min={}, max={}", min_count, max_count)),
        ("Repetition Test", rep_pass, format!("max {} consecutive", max_cons)),
        ("Adaptive Proportion", apt_pass, format!("worst {}/512", worst_count)),
        ("Runs Test", runs_pass, format!("{} runs", runs)),
        (
            "Longest Run Test",
            longest_result.pass,
            format!("longest {} bits", longest_result.longest_run),
        ),
    ];

    let passed = results.iter().filter(|(_, ok, _)| *ok).count();
    let total = results.len();

    out.push_str("\n  ── Summary ──\n");
    for (name, ok, detail) in &results {
        let icon = if *ok { "✓" } else { "✗" };
        out.push_str(&format!(
            "  {} {}: {} [{}]\n",
            icon,
            name,
            detail,
            if *ok { "PASS" } else { "FAIL" }
        ));
    }
    out.push_str(&format!(
        "  Result: {}/{} tests passed {}\n",
        passed,
        total,
        if passed == total { "✅" } else { "⚠️" }
    ));

    out
}

/// Run the same statistical analysis on Prism keystream.
fn analyze_prism_keystream() -> String {
    let mut out = String::new();

    out.push_str(&format!("\n{:=^60}\n", ""));
    out.push_str(" PRISM KEYSTREAM STATISTICAL ANALYSIS\n");
    out.push_str(&format!("{:=^60}\n", ""));

    out.push_str("\nGenerating 1MB keystream via KelvinPrism (OTP Key Generator)...\n");
    let keystream = generate_prism_keystream();
    out.push_str(&format!("  Keystream size: {} bytes\n", keystream.len()));

    // ── Shannon Entropy ────────────────────────────────────────────────────
    let shannon = compute_shannon_entropy(&keystream);
    out.push_str(&format!("  Shannon Entropy: {:.4} bits/byte (max 8.0)\n", shannon));
    if shannon > 7.5 {
        out.push_str("  [PASS] Near-maximal entropy (good randomness)\n");
    } else if shannon > 6.0 {
        out.push_str("  [WARN] Moderate entropy\n");
    } else {
        out.push_str("  [FAIL] Low entropy\n");
    }

    // ── Correlation Coefficient ─────────────────────────────────────────────
    let corr = compute_correlation(&keystream);
    out.push_str(&format!("  Adjacent-byte Correlation: {:.6} (expected ~0)\n", corr));
    if corr.abs() < 0.01 {
        out.push_str("  [PASS] No significant correlation detected\n");
    } else if corr.abs() < 0.05 {
        out.push_str("  [WARN] Weak correlation detected\n");
    } else {
        out.push_str("  [FAIL] Strong correlation detected\n");
    }

    // ── Byte value distribution (chi-square test) ──────────────────────────
    let mut counts = [0u64; 256];
    for &b in &keystream {
        counts[b as usize] += 1;
    }
    let min_count = *counts.iter().min().unwrap_or(&0);
    let max_count = *counts.iter().max().unwrap_or(&0);
    let expected = keystream.len() as f64 / 256.0;
    let missing: usize = counts.iter().filter(|&&c| c == 0).count();
    let chi_square: f64 = counts
        .iter()
        .map(|&c| {
            let diff = c as f64 - expected;
            diff * diff / expected
        })
        .sum();
    out.push_str(&format!(
        "  Byte value distribution: min={}, max={}, expected={:.0}, χ²={:.1}\n",
        min_count, max_count, expected, chi_square
    ));
    if missing > 0 {
        out.push_str(&format!("  [FAIL] {} byte values never appear in keystream\n", missing));
    } else if chi_square < 310.0 {
        out.push_str(&format!("  [PASS] Chi-square = {:.1} (critical: 310, df=255)\n", chi_square));
    } else {
        out.push_str(&format!(
            "  [FAIL] Chi-square = {:.1} exceeds critical value 310\n",
            chi_square
        ));
    }

    // ── SP 800-90B Entropy Health Tests ────────────────────────────────────
    out.push_str("\n  ── SP 800-90B Entropy Health Tests ──\n");

    let (rep_pass, max_cons) = repetition_test(&keystream);
    out.push_str(&format!(
        "  [{}] Repetition Test (max {} consecutive identical bytes)\n",
        if rep_pass { "PASS" } else { "FAIL" },
        max_cons
    ));

    let (apt_pass, worst_count, worst_off) = adaptive_proportion_test(&keystream, 512);
    out.push_str(&format!(
        "  [{}] Adaptive Proportion Test (worst window: {}/512 at offset {})\n",
        if apt_pass { "PASS" } else { "FAIL" },
        worst_count,
        worst_off
    ));

    let (runs_pass, runs, expected_runs) = runs_test_bit_level(&keystream);
    out.push_str(&format!(
        "  [{}] Runs Test ({} runs, expected ~{})\n",
        if runs_pass { "PASS" } else { "FAIL" },
        runs,
        expected_runs
    ));

    let longest_result = longest_run_bit_test(&keystream);
    out.push_str(&format!(
        "  [{}] Longest Run Test ({})\n",
        if longest_result.pass { "PASS" } else { "FAIL" },
        longest_result.detail
    ));

    // ── Summary ────────────────────────────────────────────────────────────
    let shannon_pass = shannon > 7.5;
    let corr_pass = corr.abs() < 0.01;
    let dist_pass = missing == 0 && chi_square < 310.0;

    let results = [
        ("Shannon Entropy", shannon_pass, format!("{:.4} bits/byte", shannon)),
        ("Correlation", corr_pass, format!("{:.6}", corr)),
        ("Byte Distribution", dist_pass, format!("min={}, max={}", min_count, max_count)),
        ("Repetition Test", rep_pass, format!("max {} consecutive", max_cons)),
        ("Adaptive Proportion", apt_pass, format!("worst {}/512", worst_count)),
        ("Runs Test", runs_pass, format!("{} runs", runs)),
        (
            "Longest Run Test",
            longest_result.pass,
            format!("longest {} bits", longest_result.longest_run),
        ),
    ];

    let passed = results.iter().filter(|(_, ok, _)| *ok).count();
    let total = results.len();

    out.push_str("\n  ── Summary ──\n");
    for (name, ok, detail) in &results {
        let icon = if *ok { "✓" } else { "✗" };
        out.push_str(&format!(
            "  {} {}: {} [{}]\n",
            icon,
            name,
            detail,
            if *ok { "PASS" } else { "FAIL" }
        ));
    }
    out.push_str(&format!(
        "  Result: {}/{} tests passed {}\n",
        passed,
        total,
        if passed == total { "✅" } else { "⚠️" }
    ));

    out
}

// ── Key entropy analysis (returns report string) ──────────────────────────────

#[derive(serde::Deserialize)]
struct KeyData {
    bodies: Vec<BodyData>,
}

#[derive(serde::Deserialize)]
struct BodyData {
    mass: f64,
    position: Vec3Data,
    velocity: Vec3Data,
}

#[derive(serde::Deserialize)]
struct Vec3Data {
    x: f64,
    y: f64,
    z: f64,
}

fn analyze_key_entropy(dir: &str, num_keys: usize) -> String {
    let mut out = String::new();

    out.push_str(&format!("\n{:=^60}\n", ""));
    out.push_str(&format!(" KELVIN ENTROPY ANALYSIS: {} KEYS\n", num_keys));
    out.push_str(&format!("{:=^60}\n", ""));

    let mut sun_masses: Vec<f64> = Vec::new();
    let mut planet_masses: Vec<f64> = Vec::new();
    let mut positions: Vec<(f64, f64, f64)> = Vec::new();
    let mut velocities: Vec<(f64, f64, f64)> = Vec::new();

    for i in 0..num_keys {
        let path = format!("{}/key_{:05}.json", dir, i);
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("  [WARN] Could not read {}: {}", path, e);
                continue;
            },
        };
        let key_data: KeyData = match serde_json::from_str(&content) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("  [WARN] Could not parse {}: {}", path, e);
                continue;
            },
        };

        if key_data.bodies.is_empty() {
            continue;
        }

        // Sun (first body)
        sun_masses.push(key_data.bodies[0].mass);

        // Planets (remaining bodies)
        for body in key_data.bodies.iter().skip(1) {
            planet_masses.push(body.mass);
            positions.push((body.position.x, body.position.y, body.position.z));
            velocities.push((body.velocity.x, body.velocity.y, body.velocity.z));
        }
    }

    fn print_stat<T: std::fmt::Debug + PartialOrd>(label: &str, values: &[T]) -> (String, usize) {
        let mut s = String::new();
        let n = values.len();
        let unique_count = if n == 0 {
            0
        } else {
            let mut sorted: Vec<&T> = values.iter().collect();
            sorted.sort_by(|a, b| {
                a.partial_cmp(b)
                    .expect("Incomparable values (e.g. NaN) detected during entropy analysis")
            });
            let mut count = 1;
            for i in 1..n {
                if sorted[i] != sorted[i - 1] {
                    count += 1;
                }
            }
            count
        };

        let empirical_bits = if unique_count > 0 { (unique_count as f64).log2() } else { 0.0 };
        s.push_str(&format!("\n[ {} ]\n", label));
        s.push_str(&format!("  Samples:    {}\n", n));
        s.push_str(&format!(
            "  Unique:     {} ({:.2}%)\n",
            unique_count,
            if n > 0 { unique_count as f64 / n as f64 * 100.0 } else { 0.0 }
        ));
        s.push_str(&format!("  Sample Entropy: ~{:.2} bits\n", empirical_bits));
        (s, unique_count)
    }

    let (sun_str, sun_unique) = print_stat("Sun Mass", &sun_masses);
    out.push_str(&sun_str);
    let (planet_str, planet_unique) = print_stat("Planet Masses", &planet_masses);
    out.push_str(&planet_str);
    let (pos_str, pos_unique) = print_stat("Position Vectors (3D)", &positions);
    out.push_str(&pos_str);
    let (vel_str, vel_unique) = print_stat("Velocity Vectors (3D)", &velocities);
    out.push_str(&vel_str);

    // Collision check (sort and dedup for f64)
    let mut sorted_suns = sun_masses.clone();
    sorted_suns.sort_by(|a, b| a.total_cmp(b));
    sorted_suns.dedup();
    let duplicates = sun_masses.len() - sorted_suns.len();

    out.push_str(&format!("\n{:=^60}\n", ""));
    if duplicates == 0 {
        out.push_str(" SUCCESS: No collisions detected in sun mass or configurations.\n");
    } else {
        out.push_str(&format!(" WARNING: {} collisions detected in sun mass!\n", duplicates));
    }
    out.push_str(&format!("{:=^60}\n", ""));

    // ── Summary ────────────────────────────────────────────────────────────
    // unique counts are already computed by print_stat calls above
    let sun_ok = sun_unique == sun_masses.len();
    let planet_ok = planet_unique == planet_masses.len();
    let pos_ok = pos_unique == positions.len();
    let vel_ok = vel_unique == velocities.len();
    let coll_ok = duplicates == 0;

    let key_results = [
        (
            "Sun Mass",
            sun_ok,
            format!(
                "{} samples, {} unique ({}%)",
                sun_masses.len(),
                sun_unique,
                if sun_masses.is_empty() {
                    0
                } else {
                    (sun_unique as f64 / sun_masses.len() as f64 * 100.0) as usize
                }
            ),
        ),
        (
            "Planet Masses",
            planet_ok,
            format!(
                "{} samples, {} unique ({}%)",
                planet_masses.len(),
                planet_unique,
                if planet_masses.is_empty() {
                    0
                } else {
                    (planet_unique as f64 / planet_masses.len() as f64 * 100.0) as usize
                }
            ),
        ),
        (
            "Positions",
            pos_ok,
            format!(
                "{} samples, {} unique ({}%)",
                positions.len(),
                pos_unique,
                if positions.is_empty() {
                    0
                } else {
                    (pos_unique as f64 / positions.len() as f64 * 100.0) as usize
                }
            ),
        ),
        (
            "Velocities",
            vel_ok,
            format!(
                "{} samples, {} unique ({}%)",
                velocities.len(),
                vel_unique,
                if velocities.is_empty() {
                    0
                } else {
                    (vel_unique as f64 / velocities.len() as f64 * 100.0) as usize
                }
            ),
        ),
        (
            "Collisions",
            coll_ok,
            if coll_ok {
                "none detected".to_string()
            } else {
                format!("{} collisions", duplicates)
            },
        ),
    ];

    let key_passed = key_results.iter().filter(|(_, ok, _)| *ok).count();
    let key_total = key_results.len();

    out.push_str("\n  ── Summary ──\n");
    out.push_str(&format!("  Mode: Key Entropy Analysis ({} keys)\n", num_keys));
    for (name, ok, detail) in &key_results {
        let icon = if *ok { "✓" } else { "✗" };
        out.push_str(&format!(
            "    {} {}: {} [{}]\n",
            icon,
            name,
            detail,
            if *ok { "PASS" } else { "FAIL" }
        ));
    }
    out.push_str(&format!(
        "  Result: {}/{} checks passed {}\n",
        key_passed,
        key_total,
        if key_passed == key_total { "✅" } else { "⚠️" }
    ));

    out
}

// ── Output saving ─────────────────────────────────────────────────────────────

fn save_output(output_path: &str, buffer: &str) {
    if let Some(parent) = Path::new(output_path).parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).expect("Failed to create output directory");
        }
    }
    fs::write(output_path, buffer).expect("Failed to write output file");
    println!("\nResults saved to: {}", output_path);
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    let args = parse_args();

    // Determine output path: --output flag, or auto-generate timestamped filename
    let output_path = args.output.clone().unwrap_or_else(|| {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("test_results/entropy_report_{}.txt", ts)
    });

    let mut report = String::new();

    // Only run key generation if --keys was explicitly provided
    let has_keys_flag = std::env::args().any(|a| a == "--keys");

    if has_keys_flag && args.num_keys > 0 {
        let dir = Path::new(&args.dir);
        if !dir.exists() {
            fs::create_dir_all(dir).expect("Failed to create output directory");
        }

        println!("Starting generation of {} keys (Level: {})...", args.num_keys, args.level);
        for i in 0..args.num_keys {
            if i > 0 && i % 50 == 0 {
                println!("  ... {}/{} complete", i, args.num_keys);
            }
            run_keygen(i, &args.dir, &args.level);
        }

        let key_report = analyze_key_entropy(&args.dir, args.num_keys);
        print!("{}", key_report);
        report.push_str(&key_report);
    }

    if args.keystream {
        let ks_report = analyze_keystream();
        print!("{}", ks_report);
        report.push_str(&ks_report);

        let prism_report = analyze_prism_keystream();
        print!("{}", prism_report);
        report.push_str(&prism_report);
    }

    // Save output to file
    if !report.is_empty() {
        save_output(&output_path, &report);
    }
}

fn run_keygen(index: usize, output_dir: &str, level: &str) {
    let output_path = format!("{}/key_{:05}.json", output_dir, index);
    let status = std::process::Command::new("cargo")
        .args([
            "run",
            "--release",
            "-p",
            "kelvin-cli",
            "--",
            "keygen",
            "--level",
            level,
            "--output",
            &output_path,
        ])
        .status()
        .expect("Failed to run keygen");
    if !status.success() {
        eprintln!("Warning: keygen failed for key {}", index);
    }
}
