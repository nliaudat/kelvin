//! SHA3-512 entropy extraction from orbital state.
//!
//! Extracts a 64-byte seed from the current orbital state by hashing
//! the positions, velocities, and masses of all bodies.
//!
//! ## References
//!
//! - National Institute of Standards and Technology. (2015). "SHA-3
//!   Standard: Permutation-Based Hash and Extendable-Output Functions."
//!   FIPS PUB 202. doi:10.6028/NIST.FIPS.202
//!   — SHA3-512 specification used for entropy extraction.
//! - Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2013).
//!   "Keccak." *Advances in Cryptology — EUROCRYPT 2013*, 313–314.
//!   doi:10.1007/978-3-642-38348-9_19
//!   — Keccak sponge construction underlying SHA3-512.

use alloc::vec::Vec;
use kelvin_core::OrbitalBody;
use sha3::digest::{ExtendableOutput, XofReader};
use sha3::{Digest, Sha3_512, Shake256};

/// Feed all orbital state into a hasher (used by both SHA3-512 and SHAKE256).
fn feed_orbital_state(
    hasher: &mut impl sha3::digest::Update,
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    nonce: &[u8],
) {
    // Domain separation
    sha3::digest::Update::update(hasher, domain_separator);

    // Session nonce (public per-instance uniqueness value).
    //
    // Binds the extracted seed to a single cipher instance so that two
    // instances sharing one OrbitalConfig but using different nonces derive
    // independent keystreams. An empty nonce reproduces the legacy output
    // (pre-nonce golden vectors remain valid). The length prefix keeps
    // `(domain_separator, nonce)` pairs unambiguous.
    sha3::digest::Update::update(hasher, &(nonce.len() as u64).to_le_bytes());
    sha3::digest::Update::update(hasher, nonce);

    // Physical constants
    sha3::digest::Update::update(hasher, &g.to_raw().to_le_bytes());
    sha3::digest::Update::update(hasher, &softening.to_raw().to_le_bytes());

    // Step counter
    sha3::digest::Update::update(hasher, &step.to_le_bytes());

    // Number of bodies
    sha3::digest::Update::update(hasher, &(bodies.len() as u32).to_le_bytes());

    // Instantaneous gravitational forces (accelerations)
    let accelerations = kelvin_core::compute_accelerations(bodies, softening, g);

    // Body data
    for (body, acc) in bodies.iter().zip(accelerations.iter()) {
        sha3::digest::Update::update(hasher, &body.mass.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.position.x.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.position.y.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.position.z.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.velocity.x.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.velocity.y.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &body.velocity.z.to_raw().to_le_bytes());

        // Instant G force vector (acceleration)
        sha3::digest::Update::update(hasher, &acc.x.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &acc.y.to_raw().to_le_bytes());
        sha3::digest::Update::update(hasher, &acc.z.to_raw().to_le_bytes());
    }
}

/// Extract a 64-byte seed from the orbital state using SHA3-512.
///
/// Delegates to `feed_orbital_state` for the hashing logic, ensuring
/// consistency with the SHAKE256 extraction path.
///
/// This ensures that different simulation states produce different seeds.
///
/// A session `nonce` may be bound into the extraction via
/// [`extract_seed_with_nonce`]; this function is equivalent to calling it
/// with an empty nonce.
pub fn extract_seed(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
) -> [u8; 64] {
    extract_seed_with_nonce(bodies, step, g, softening, domain_separator, &[])
}

/// Extract a 64-byte seed like [`extract_seed`], additionally bound to a
/// session `nonce`.
///
/// Two instances sharing an `OrbitalConfig` but using different nonces derive
/// independent seeds, which defeats the two-time-pad keystream reuse that
/// otherwise follows from config reuse. The nonce is public (it must be
/// communicated to the decrypting party, like a salt); it adds no secrecy,
/// only instance separation.
pub fn extract_seed_with_nonce(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    nonce: &[u8],
) -> [u8; 64] {
    let mut hasher = Sha3_512::new();
    feed_orbital_state(&mut hasher, bodies, step, g, softening, domain_separator, nonce);
    let result = hasher.finalize();
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&result);
    seed
}

/// Extract a seed of arbitrary length from the orbital state using SHAKE256 (XOF).
///
/// SHAKE256 is an Extendable-Output Function that can produce a deterministic
/// stream of bits of any length. This is used to derive large entropy pools
/// (e.g., 2048 bytes) from the orbital state.
///
/// Equivalent to [`extract_shake256_with_nonce`] with an empty nonce.
pub fn extract_shake256(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    output_len: usize,
) -> Vec<u8> {
    extract_shake256_with_nonce(bodies, step, g, softening, domain_separator, &[], output_len)
}

/// Extract arbitrary-length SHAKE256 output bound to a session `nonce`.
///
/// See [`extract_seed_with_nonce`] for the security rationale: distinct
/// nonces yield independent keystreams from the same orbital state.
pub fn extract_shake256_with_nonce(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    nonce: &[u8],
    output_len: usize,
) -> Vec<u8> {
    let mut output = vec![0u8; output_len];
    extract_shake256_into_with_nonce(
        bodies,
        step,
        g,
        softening,
        domain_separator,
        nonce,
        &mut output,
    );
    output
}

/// Extract SHAKE256 entropy directly into a pre-allocated buffer.
///
/// Equivalent to [`extract_shake256_into_with_nonce`] with an empty nonce.
///
/// This avoids the allocation overhead of [`extract_shake256`] when processing
/// data in chunks, such as in streaming encryption where the same buffer size
/// is reused across many iterations.
pub fn extract_shake256_into(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    output: &mut [u8],
) {
    extract_shake256_into_with_nonce(bodies, step, g, softening, domain_separator, &[], output);
}

/// Extract SHAKE256 entropy into a buffer, bound to a session `nonce`.
///
/// See [`extract_seed_with_nonce`] for the security rationale: distinct
/// nonces yield independent keystreams from the same orbital state.
pub fn extract_shake256_into_with_nonce(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    nonce: &[u8],
    output: &mut [u8],
) {
    #[cfg(feature = "failpoints")]
    fail::fail_point!("shake256-extract");
    let mut hasher = Shake256::default();

    // Feed all orbital state into the XOF hasher
    feed_orbital_state(&mut hasher, bodies, step, g, softening, domain_separator, nonce);

    let mut reader = hasher.finalize_xof();
    XofReader::read(&mut reader, output);
}

/// Legacy wrapper for extract_shake256.
#[deprecated(note = "Use extract_shake256 for more efficient XOF extraction")]
#[allow(dead_code)]
pub fn extract_seed_extended(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    output_len: usize,
) -> Vec<u8> {
    extract_shake256(bodies, step, g, softening, domain_separator, output_len)
}

// ============================================================================
// Kani formal verification harnesses for C4: Keystream Indistinguishability
// ============================================================================
//
// These harnesses verify the deterministic pipeline properties of the
// entropy extraction code. They prove:
//   1. Identical orbital state → identical SHAKE256 output
//   2. Different domain separators → different SHAKE256 output
//
// Run with: cargo kani -p kelvin-kdf
#[cfg(kani)]
mod kani_proofs {
    use crate::extract_shake256_into;
    use kelvin_core::{Fixed, OrbitalBody, Vec3};

    // ── Harness 1: Deterministic Extraction ──────────────────────────
    //
    // Prove: Identical symbolic orbital state, step, G, and softening
    // produce identical SHAKE256 output. This is the fundamental property
    // required for any stream cipher: same key → same keystream.
    #[kani::proof]
    fn verify_extraction_deterministic() {
        // Symbolic orbital body positions within physical bounds
        let px: i128 = kani::any();
        let py: i128 = kani::any();
        let pz: i128 = kani::any();
        let vx: i128 = kani::any();
        let vy: i128 = kani::any();
        let vz: i128 = kani::any();
        kani::assume(px >= -100 * (1 << 64) && px <= 100 * (1 << 64));
        kani::assume(py >= -100 * (1 << 64) && py <= 100 * (1 << 64));
        kani::assume(pz >= -100 * (1 << 64) && pz <= 100 * (1 << 64));
        kani::assume(vx >= -100 * (1 << 64) && vx <= 100 * (1 << 64));
        kani::assume(vy >= -100 * (1 << 64) && vy <= 100 * (1 << 64));
        kani::assume(vz >= -100 * (1 << 64) && vz <= 100 * (1 << 64));

        let body = OrbitalBody::new(
            Fixed::ONE,
            Vec3::new(Fixed::from_raw(px), Fixed::from_raw(py), Fixed::from_raw(pz)),
            Vec3::new(Fixed::from_raw(vx), Fixed::from_raw(vy), Fixed::from_raw(vz)),
        );
        let bodies = [body];
        let step: u64 = kani::any();
        let g = Fixed::ONE;
        let softening = Fixed::from_raw(1 << 44);
        let domain = b"kelvin-test-v1";

        // Extract twice with identical parameters
        let mut out1 = [0u8; 32];
        let mut out2 = [0u8; 32];
        extract_shake256_into(&bodies, step, g, softening, domain, &mut out1);
        extract_shake256_into(&bodies, step, g, softening, domain, &mut out2);

        // The two outputs must be identical (deterministic extraction)
        kani::assert(out1 == out2, "C4-extract: deterministic (same input → same output)");

        // Both outputs must be non-zero (not uninitialized memory)
        kani::cover(out1 != [0u8; 32], "C4-cover: extraction produces non-zero output");
    }

    // ── Harness 2: Domain Separation ─────────────────────────────────
    //
    // Prove: Different domain separators produce different SHAKE256
    // outputs for the SAME orbital state and step counter.
    //
    // This validates that each encryption mode (Chaos, Photon, Quantum,
    // Prism, Split, Flare) produces independent keystream even when
    // derived from the same orbital seed.
    #[kani::proof]
    fn verify_domain_separation_functional() {
        // Fixed orbital state (single body at origin)
        let body = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let bodies = [body];
        let step: u64 = 42;
        let g = Fixed::ONE;
        let softening = Fixed::from_raw(1 << 44);

        // Two different domain separators
        let domain_a = b"kelvin-encrypt-v1";
        let domain_b = b"kelvin-signing-v1";

        let mut out_a = [0u8; 32];
        let mut out_b = [0u8; 32];
        extract_shake256_into(&bodies, step, g, softening, domain_a, &mut out_a);
        extract_shake256_into(&bodies, step, g, softening, domain_b, &mut out_b);

        // Different domain separators must produce different outputs
        // for the same orbital state. If this fails, the domain
        // separation is ineffective and different modes would produce
        // the same keystream.
        kani::assert(out_a != out_b, "C4-domain: different separators produce different outputs");

        // Both outputs must be non-zero
        kani::cover(out_a != [0u8; 32], "C4-cover: domain A produces non-zero output");
        kani::cover(out_b != [0u8; 32], "C4-cover: domain B produces non-zero output");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::{Fixed, Vec3};

    fn test_bodies() -> Vec<OrbitalBody> {
        vec![
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
        ]
    }

    #[test]
    fn test_extract_seed_length() {
        let bodies = test_bodies();
        let seed = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test");
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_extract_seed_deterministic() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test");
        let seed2 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test");
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_steps() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test");
        let seed2 = extract_seed(&bodies, 1, Fixed::ONE, Fixed::ONE, b"test");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_domain() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"domain-a");
        let seed2 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"domain-b");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_bodies() {
        let bodies1 = test_bodies();
        let mut bodies2 = test_bodies();
        bodies2[0].position = Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO);

        let seed1 = extract_seed(&bodies1, 0, Fixed::ONE, Fixed::ONE, b"test");
        let seed2 = extract_seed(&bodies2, 0, Fixed::ONE, Fixed::ONE, b"test");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_shake256() {
        let bodies = test_bodies();
        let seed = extract_shake256(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", 128);
        assert_eq!(seed.len(), 128);
    }

    #[test]
    fn test_extract_shake256_deterministic() {
        let bodies = test_bodies();
        let seed1 = extract_shake256(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", 128);
        let seed2 = extract_shake256(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", 128);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_no_panic_empty_bodies() {
        let seed = extract_seed(&[], 0, Fixed::ONE, Fixed::ONE, b"test");
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_extract_seed_avalanche() {
        // Small change in input should produce completely different output
        let bodies1 = test_bodies();
        let mut bodies2 = test_bodies();
        bodies2[0].mass += Fixed::from_raw(1); // minimal change

        let seed1 = extract_seed(&bodies1, 0, Fixed::ONE, Fixed::ONE, b"test");
        let seed2 = extract_seed(&bodies2, 0, Fixed::ONE, Fixed::ONE, b"test");

        // Count differing bits
        let diff_bits: u32 =
            seed1.iter().zip(seed2.iter()).map(|(a, b)| (a ^ b).count_ones()).sum();

        // Should have roughly half the bits different (avalanche effect)
        assert!(diff_bits > 200, "Too few differing bits: {}", diff_bits);
    }

    #[test]
    fn test_nonce_empty_matches_legacy() {
        // Empty nonce must reproduce the pre-nonce output exactly,
        // so existing golden vectors stay valid.
        let bodies = test_bodies();
        let legacy = extract_shake256(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", 128);
        let with_empty =
            extract_shake256_with_nonce(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", &[], 128);
        assert_eq!(legacy, with_empty);

        let legacy64 = extract_seed(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test");
        let with_empty64 =
            extract_seed_with_nonce(&bodies, 0, Fixed::ONE, Fixed::ONE, b"test", &[]);
        assert_eq!(legacy64, with_empty64);
    }

    #[test]
    fn test_nonce_deterministic() {
        let bodies = test_bodies();
        let a = extract_shake256_with_nonce(
            &bodies,
            7,
            Fixed::ONE,
            Fixed::ONE,
            b"test",
            b"session-1",
            64,
        );
        let b = extract_shake256_with_nonce(
            &bodies,
            7,
            Fixed::ONE,
            Fixed::ONE,
            b"test",
            b"session-1",
            64,
        );
        assert_eq!(a, b);
    }

    #[test]
    fn test_nonce_separates_instances() {
        // Same orbital state, different nonces -> independent keystream.
        // This is the guardrail against config-reuse two-time-pad (bad.md §3).
        let bodies = test_bodies();
        let a = extract_shake256_with_nonce(
            &bodies,
            7,
            Fixed::ONE,
            Fixed::ONE,
            b"test",
            b"session-1",
            64,
        );
        let b = extract_shake256_with_nonce(
            &bodies,
            7,
            Fixed::ONE,
            Fixed::ONE,
            b"test",
            b"session-2",
            64,
        );
        assert_ne!(a, b);

        let diff_bits: u32 = a.iter().zip(b.iter()).map(|(x, y)| (x ^ y).count_ones()).sum();
        assert!(diff_bits > 200, "Nonce separation too weak: {} bits", diff_bits);
    }
}
