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
use sha3::{Digest, Sha3_512};
use kelvin_core::{Fixed, OrbitalBody};

/// Extract a 64-byte seed from the orbital state using SHA3-512.
///
/// The input to the hash is:
/// - Domain separator (personalization string)
/// - For each body: mass, position (x, y, z), velocity (x, y, z)
/// - Step counter
///
/// This ensures that different simulation states produce different seeds.
pub fn extract_seed(
    bodies: &[OrbitalBody],
    step: u64,
    domain_separator: &[u8],
) -> [u8; 64] {
    let mut hasher = Sha3_512::new();

    // Domain separation
    hasher.update(domain_separator);

    // Step counter
    hasher.update(&step.to_le_bytes());

    // Number of bodies
    hasher.update(&(bodies.len() as u32).to_le_bytes());

    // Body data
    for body in bodies {
        hasher.update(&body.mass.to_raw().to_le_bytes());
        hasher.update(&body.position.x.to_raw().to_le_bytes());
        hasher.update(&body.position.y.to_raw().to_le_bytes());
        hasher.update(&body.position.z.to_raw().to_le_bytes());
        hasher.update(&body.velocity.x.to_raw().to_le_bytes());
        hasher.update(&body.velocity.y.to_raw().to_le_bytes());
        hasher.update(&body.velocity.z.to_raw().to_le_bytes());
    }

    let result = hasher.finalize();
    let mut seed = [0u8; 64];
    seed.copy_from_slice(&result);
    seed
}

/// Extract a seed of arbitrary length from the orbital state.
///
/// Uses SHA3-512 in counter mode to generate the requested number of bytes.
pub fn extract_seed_extended(
    bodies: &[OrbitalBody],
    step: u64,
    domain_separator: &[u8],
    output_len: usize,
) -> Vec<u8> {
    let mut output = Vec::with_capacity(output_len);
    let mut counter: u64 = 0;

    while output.len() < output_len {
        let mut hasher = Sha3_512::new();
        hasher.update(domain_separator);
        hasher.update(&step.to_le_bytes());
        hasher.update(&counter.to_le_bytes());
        hasher.update(&(bodies.len() as u32).to_le_bytes());

        for body in bodies {
            hasher.update(&body.mass.to_raw().to_le_bytes());
            hasher.update(&body.position.x.to_raw().to_le_bytes());
            hasher.update(&body.position.y.to_raw().to_le_bytes());
            hasher.update(&body.position.z.to_raw().to_le_bytes());
            hasher.update(&body.velocity.x.to_raw().to_le_bytes());
            hasher.update(&body.velocity.y.to_raw().to_le_bytes());
            hasher.update(&body.velocity.z.to_raw().to_le_bytes());
        }

        let result = hasher.finalize();
        let remaining = output_len - output.len();
        let to_copy = remaining.min(64);
        output.extend_from_slice(&result[..to_copy]);
        counter += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::{Fixed, Vec3};

    fn test_bodies() -> Vec<OrbitalBody> {
        vec![
            OrbitalBody::new(
                Fixed::ONE,
                Vec3::ZERO,
                Vec3::ZERO,
            ),
            OrbitalBody::new(
                Fixed::from_raw(1 << 54),
                Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
                Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
            ),
        ]
    }

    #[test]
    fn test_extract_seed_length() {
        let bodies = test_bodies();
        let seed = extract_seed(&bodies, 0, b"test");
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_extract_seed_deterministic() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, b"test");
        let seed2 = extract_seed(&bodies, 0, b"test");
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_steps() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, b"test");
        let seed2 = extract_seed(&bodies, 1, b"test");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_domain() {
        let bodies = test_bodies();
        let seed1 = extract_seed(&bodies, 0, b"domain-a");
        let seed2 = extract_seed(&bodies, 0, b"domain-b");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_different_bodies() {
        let bodies1 = test_bodies();
        let mut bodies2 = test_bodies();
        bodies2[0].position = Vec3::new(Fixed::from_int(1), Fixed::ZERO, Fixed::ZERO);

        let seed1 = extract_seed(&bodies1, 0, b"test");
        let seed2 = extract_seed(&bodies2, 0, b"test");
        assert_ne!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_extended() {
        let bodies = test_bodies();
        let seed = extract_seed_extended(&bodies, 0, b"test", 128);
        assert_eq!(seed.len(), 128);
    }

    #[test]
    fn test_extract_seed_extended_deterministic() {
        let bodies = test_bodies();
        let seed1 = extract_seed_extended(&bodies, 0, b"test", 128);
        let seed2 = extract_seed_extended(&bodies, 0, b"test", 128);
        assert_eq!(seed1, seed2);
    }

    #[test]
    fn test_extract_seed_no_panic_empty_bodies() {
        let seed = extract_seed(&[], 0, b"test");
        assert_eq!(seed.len(), 64);
    }

    #[test]
    fn test_extract_seed_avalanche() {
        // Small change in input should produce completely different output
        let bodies1 = test_bodies();
        let mut bodies2 = test_bodies();
        bodies2[0].mass = bodies2[0].mass + Fixed::from_raw(1); // minimal change

        let seed1 = extract_seed(&bodies1, 0, b"test");
        let seed2 = extract_seed(&bodies2, 0, b"test");

        // Count differing bits
        let diff_bits: u32 = seed1.iter().zip(seed2.iter())
            .map(|(a, b)| (a ^ b).count_ones())
            .sum();

        // Should have roughly half the bits different (avalanche effect)
        assert!(diff_bits > 200, "Too few differing bits: {}", diff_bits);
    }
}
