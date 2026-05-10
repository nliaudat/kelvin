//! Asymmetric key derivation for Kelvin using Curve25519.
//!
//! Enables deriving Curve25519 public/private key pairs from orbital chaos.
//!
//! ## References
//!
//! - Bernstein, D. J. (2006). "Curve25519: New Diffie-Hellman Speed Records."
//!   *Public Key Cryptography — PKC 2006*, 335–352.
//!   doi:10.1007/11745853_21
//! - Langley, A., Hamburg, M., & Turner, S. (2016). "Elliptic Curves for Security."
//!   RFC 7748. doi:10.17487/RFC7748

use curve25519_dalek::scalar::Scalar;
use x25519_dalek::{PublicKey, StaticSecret};
use crate::{OrbitalConfig, extract_seed};
use kelvin_core::{OrbitalBody, simulate};

/// Errors related to asymmetric key operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsymmetricError {
    /// Simulation error during key derivation.
    SimulationError,
}

/// A Curve25519 key pair derived from an orbital configuration.
#[derive(Clone)]
pub struct OrbitalKeyPair {
    /// The derived private key (StaticSecret).
    pub private_key: StaticSecret,
    /// The derived public key.
    pub public_key: PublicKey,
}

impl core::fmt::Debug for OrbitalKeyPair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrbitalKeyPair")
            .field("public_key", &self.public_key)
            .field("private_key", &"<REDACTED>")
            .finish()
    }
}

impl OrbitalKeyPair {
    /// Derive a Curve25519 key pair from an existing orbital state.
    pub fn from_bodies(bodies: &[OrbitalBody], step: u64) -> Self {
        // Extract 512 bits (64 bytes)
        let seed_512 = extract_seed(bodies, step, b"kelvin-asymmetric-v1");

        // Wide reduction to derive scalar
        // Scalar::from_bytes_mod_order_wide ensures no bias by reducing a 512-bit input.
        let scalar = Scalar::from_bytes_mod_order_wide(&seed_512);
        let secret_bytes = scalar.to_bytes();
        
        // Create X25519 StaticSecret.
        // Note: StaticSecret::from() will perform standard clamping as per RFC 7748.
        let private_key = StaticSecret::from(secret_bytes);
        let public_key = PublicKey::from(&private_key);

        OrbitalKeyPair {
            private_key,
            public_key,
        }
    }

    /// Derive a Curve25519 key pair from an orbital configuration.
    ///
    /// This method:
    /// 1. Runs the orbital simulation as specified in the config.
    /// 2. Extracts 512 bits of entropy from the final state.
    /// 3. Performs wide reduction and computes the key pair.
    pub fn derive(config: &OrbitalConfig) -> Result<Self, AsymmetricError> {
        let mut bodies = config.bodies.clone();
        
        // Run simulation
        simulate(
            &mut bodies,
            config.total_steps,
            config.dt,
            config.softening,
            config.g,
        );

        Ok(Self::from_bodies(&bodies, config.total_steps))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::{Fixed, Vec3, OrbitalBody};

    fn test_config() -> OrbitalConfig {
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
        
        OrbitalConfig::new(
            vec![sun, planet1, planet2],
            50,
            10,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        ).unwrap()
    }

    #[test]
    fn test_asymmetric_derivation_deterministic() {
        let config = test_config();
        
        let kp1 = OrbitalKeyPair::derive(&config).unwrap();
        let kp2 = OrbitalKeyPair::derive(&config).unwrap();
        
        assert_eq!(kp1.private_key.to_bytes(), kp2.private_key.to_bytes());
        assert_eq!(kp1.public_key.as_bytes(), kp2.public_key.as_bytes());
    }

    #[test]
    fn test_asymmetric_derivation_different_configs() {
        let config1 = test_config();
        let mut config2 = test_config();
        config2.total_steps += 1;
        
        let kp1 = OrbitalKeyPair::derive(&config1).unwrap();
        let kp2 = OrbitalKeyPair::derive(&config2).unwrap();
        
        assert_ne!(kp1.private_key.to_bytes(), kp2.private_key.to_bytes());
        assert_ne!(kp1.public_key.as_bytes(), kp2.public_key.as_bytes());
    }
}
