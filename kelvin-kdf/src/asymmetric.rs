//! Hybrid Post-Quantum asymmetric key derivation for Kelvin.
//!
//! Enables deriving hybrid key pairs from orbital chaos, combining:
//! - **ML-DSA-65**: FIPS 204 standardized digital signatures (Primary Identity)
//! - **ML-KEM-768**: FIPS 203 standardized key encapsulation
//! - **Curve25519**: Classical elliptic curve Diffie-Hellman
//!
//! ## References
//!
//! - NIST (2024). "FIPS 203: Module-Lattice-Based Key-Encapsulation Mechanism Standard."
//! - NIST (2024). "FIPS 204: Module-Lattice-Based Digital Signature Standard."
//! - Bernstein, D. J. (2006). "Curve25519: New Diffie-Hellman Speed Records."
//!   *Public Key Cryptography — PKC 2006*, 335–352.
//!   doi:10.1007/11745853_21
//! - Langley, A., Hamburg, M., & Turner, S. (2016). "Elliptic Curves for Security."
//!   RFC 7748. doi:10.17487/RFC7748

use curve25519_dalek::scalar::Scalar;
use x25519_dalek::{PublicKey, StaticSecret};
use ml_kem::{MlKem768, DecapsulationKey, EncapsulationKey};
use ml_dsa::{MlDsa65, SigningKey, VerifyingKey, Keypair};
use crate::{OrbitalConfig, extract_seed};
use kelvin_core::{OrbitalBody, simulate};

/// Errors related to asymmetric key operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AsymmetricError {
    /// Simulation error during key derivation.
    SimulationError,
    /// Configuration has not reached the chaotic regime.
    InsufficientChaos,
}

/// A Hybrid Post-Quantum key pair derived from an orbital configuration.
/// 
/// Contains:
/// - Curve25519 (Classical)
/// - ML-KEM-768 (Post-Quantum KEM)
/// - ML-DSA-65 (Post-Quantum Signature)
#[derive(Clone)]
pub struct OrbitalKeyPair {
    /// The derived Curve25519 private key.
    pub curve_private: StaticSecret,
    /// The derived Curve25519 public key.
    pub curve_public: PublicKey,
    /// The derived ML-KEM-768 decapsulation key.
    pub kem_private: DecapsulationKey<MlKem768>,
    /// The derived ML-KEM-768 encapsulation key.
    pub kem_public: EncapsulationKey<MlKem768>,
    /// The derived ML-DSA-65 signing key.
    pub dsa_private: SigningKey<MlDsa65>,
    /// The derived ML-DSA-65 verifying key.
    pub dsa_public: VerifyingKey<MlDsa65>,
}

impl core::fmt::Debug for OrbitalKeyPair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrbitalKeyPair")
            .field("curve_public", &self.curve_public)
            .field("kem_public", &self.kem_public)
            .field("dsa_public", &self.dsa_public)
            .field("private_material", &"<REDACTED>")
            .finish()
    }
}

impl OrbitalKeyPair {
    /// Derive a Hybrid key pair from an existing orbital state.
    pub fn from_bodies(bodies: &[OrbitalBody], step: u64) -> Self {
        // 1. Derive Curve25519 (Classical)
        let seed_ecc = extract_seed(bodies, step, b"kelvin-curve25519-v1");
        let scalar = Scalar::from_bytes_mod_order_wide(&seed_ecc);
        let curve_private = StaticSecret::from(scalar.to_bytes());
        let curve_public = PublicKey::from(&curve_private);

        // 2. Derive ML-KEM-768 (Post-Quantum KEM)
        let seed_kem = extract_seed(bodies, step, b"kelvin-ml-kem-v1");
        let kem_private = DecapsulationKey::<MlKem768>::from_seed(seed_kem.into());
        let kem_public = kem_private.encapsulation_key().clone();

        // 3. Derive ML-DSA-65 (Post-Quantum Signature)
        let seed_dsa = extract_seed(bodies, step, b"kelvin-ml-dsa-v1");
        let dsa_seed_32: [u8; 32] = seed_dsa[0..32].try_into().expect("SHA3-512 must be 64 bytes");
        let dsa_private = SigningKey::<MlDsa65>::from_seed(&dsa_seed_32.into());
        let dsa_public = dsa_private.verifying_key().clone();

        OrbitalKeyPair {
            curve_private,
            curve_public,
            kem_private,
            kem_public,
            dsa_private,
            dsa_public,
        }
    }

    /// Derive a Hybrid key pair from an orbital configuration.
    ///
    /// This method:
    /// 1. Verifies that the configuration has reached the chaotic regime (Lyapunov horizon).
    /// 2. Runs the orbital simulation as specified in the config.
    /// 3. Extracts entropy and computes the hybrid key pair.
    pub fn derive(config: &OrbitalConfig) -> Result<Self, AsymmetricError> {
        // Enforce Lyapunov horizon check (Safety first)
        let estimator = crate::lyapunov::LyapunovEstimator::new(
            &config.bodies,
            config.dt,
            config.softening,
            config.g,
        );
        
        let result = estimator.estimate(config.total_steps / 10, config.total_steps)
            .map_err(|_| AsymmetricError::SimulationError)?;
        
        if config.total_steps < result.safe_steps {
            return Err(AsymmetricError::InsufficientChaos);
        }

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
            50,
            10,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        ).unwrap()
    }

    #[test]
    fn test_asymmetric_derivation_deterministic() {
        use ml_kem::KeyExport;
        let config = test_config();
        
        let kp1 = OrbitalKeyPair::derive(&config).unwrap();
        let kp2 = OrbitalKeyPair::derive(&config).unwrap();
        
        assert_eq!(kp1.curve_private.to_bytes(), kp2.curve_private.to_bytes());
        assert_eq!(kp1.curve_public.as_bytes(), kp2.curve_public.as_bytes());
        
        // Verify PQ keys are deterministic
        assert_eq!(kp1.kem_public.to_bytes(), kp2.kem_public.to_bytes());
        assert_eq!(kp1.dsa_public.to_bytes(), kp2.dsa_public.to_bytes());
    }

    #[test]
    fn test_asymmetric_derivation_different_configs() {
        use ml_kem::KeyExport;
        let config1 = test_config();
        let mut config2 = test_config();
        config2.total_steps += 1;
        
        let kp1 = OrbitalKeyPair::derive(&config1).unwrap();
        let kp2 = OrbitalKeyPair::derive(&config2).unwrap();
        
        assert_ne!(kp1.curve_private.to_bytes(), kp2.curve_private.to_bytes());
        assert_ne!(kp1.kem_public.to_bytes(), kp2.kem_public.to_bytes());
    }
}
