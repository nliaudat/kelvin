//! Golden Hash Proof — End-to-End Pipeline Correctness
//!
//! This test proves that the full `simulate_and_extract_seed` pipeline produces
//! a known, deterministic output for a fixed orbital configuration. This serves
//! as a regression guard: if any algorithm change alters the keystream,
//! this test fails — alerting the developer immediately.
//!
//! Together with the Kani proofs in `proofs/kani/pipeline_proofs.rs` (which
//! verify invariant preservation, domain separation, and loop equivalence),
//! this constitutes the complete L3: Pipeline Integrity verification.
//!
//! ## Updating
//!
//! Run the test and copy the hash output from stderr.

use kelvin::{simulate_and_extract_seed, OrbitalConfig};
use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};
use sha3::{Digest, Sha3_256};

// The golden SHA3-256 hash of the orbital seed produced by
// simulate_and_extract_seed() with the deterministic config below.
// Initial capture: 2026-05-29; re-captured 2026-11-25 after intentional pipeline
// changes (session nonces, Lyapunov ln(10) fix, extractor updates).
// 5-body, 500 steps, Verlet, DEFAULT_DT
const GOLDEN_HASH: [u8; 32] = [
    0x82, 0x25, 0x72, 0xdf, 0xc8, 0xb2, 0x60, 0x88, 0xab, 0xf1, 0xa0, 0x4e, 0x91, 0x67, 0x8a, 0x97,
    0x9d, 0x66, 0xcc, 0x91, 0x49, 0xc2, 0x8d, 0x06, 0xcb, 0xa3, 0xa2, 0xee, 0xc3, 0xb5, 0x51, 0x2e,
];

fn golden_hash_config() -> OrbitalConfig {
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
    // 500 steps (above Lyapunov horizon ~271), monitor every 100 steps
    // for stable Verlet integration without body ejection
    OrbitalConfig::new(bodies, 500, 100, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
        .expect("valid golden hash config")
}

#[test]
fn test_golden_hash_pipeline() {
    let config = golden_hash_config();
    let (seed, _bodies) = simulate_and_extract_seed(&config).expect("simulate_and_extract_seed");
    let hash = Sha3_256::digest(&seed);
    let hash_bytes: [u8; 32] = hash.into();

    if hash_bytes != GOLDEN_HASH {
        eprintln!();
        eprintln!("GOLDEN HASH CAPTURE");
        eprintln!("const GOLDEN_HASH: [u8; 32] = [");
        for chunk in hash_bytes.chunks(16) {
            let row: Vec<String> = chunk.iter().map(|b| format!("0x{:02x},", b)).collect();
            eprintln!("    {}", row.join(" "));
        }
        eprintln!("];");
        eprintln!(
            "// hex: {}",
            hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
        );
        eprintln!();
        panic!("golden hash mismatch (actual hash printed above)");
    }

    assert!(seed.iter().any(|&b| b != 0), "seed should not be all zeros");
}
