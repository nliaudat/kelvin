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
) {
    // Domain separation
    sha3::digest::Update::update(hasher, domain_separator);

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
pub fn extract_seed(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
) -> [u8; 64] {
    let mut hasher = Sha3_512::new();
    feed_orbital_state(&mut hasher, bodies, step, g, softening, domain_separator);
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
pub fn extract_shake256(
    bodies: &[OrbitalBody],
    step: u64,
    g: kelvin_core::Fixed,
    softening: kelvin_core::Fixed,
    domain_separator: &[u8],
    output_len: usize,
) -> Vec<u8> {
    let mut hasher = Shake256::default();

    // Feed all orbital state into the XOF hasher
    feed_orbital_state(&mut hasher, bodies, step, g, softening, domain_separator);

    let mut output = vec![0u8; output_len];
    let mut reader = hasher.finalize_xof();
    XofReader::read(&mut reader, &mut output);
    output
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
}
