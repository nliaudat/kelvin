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
// Initial capture: 2026-05-29, 5-body, 500 steps, Verlet, DEFAULT_DT
const GOLDEN_HASH: [u8; 32] = [
    0x65, 0x2e, 0xa8, 0x9e, 0xc3, 0x71, 0x62, 0x26, 0x52, 0x29, 0x47, 0x79, 0xf2, 0x4a, 0xb8, 0xcb,
    0x15, 0xd5, 0x2a, 0xe8, 0xee, 0x56, 0x38, 0x2e, 0x20, 0x2e, 0xa4, 0x14, 0x5b, 0xee, 0x07, 0x9e,
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
        for chunk in hash_bytes.chunks(8) {
            eprint!("    ");
            for b in chunk {
                eprint!("0x{:02x}, ", b);
            }
            eprintln!();
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
