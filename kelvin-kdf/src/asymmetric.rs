//! Hybrid Post-Quantum asymmetric key derivation for Kelvin.
//!
//! Enables deriving hybrid key pairs from orbital chaos, combining:
//! - **ML-DSA-65**: FIPS 204 standardized digital signatures (Primary Identity)
//! - **ML-KEM-768**: FIPS 203 standardized key encapsulation
//! - **Curve25519**: Classical elliptic curve Diffie-Hellman
//! - **Ed25519**: Classical digital signatures (via RustCrypto `ed25519-dalek`)
//!
//! Also provides adapters for RustCrypto's `signature` crate traits,
//! enabling interoperability with the broader Rust cryptographic ecosystem.
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
//! - Bernstein, D. J., et al. (2012). "High-speed high-security signatures."
//!   *Journal of Cryptographic Engineering*, 2(2), 77–89.
//!   doi:10.1007/s13389-012-0027-1

use curve25519_dalek::scalar::Scalar;
use x25519_dalek::{PublicKey, StaticSecret};
use ml_kem::{MlKem768, DecapsulationKey, EncapsulationKey};
use ml_dsa::{MlDsa65, SigningKey, VerifyingKey, Keypair};
use ed25519_dalek::{SigningKey as EdSigningKey, VerifyingKey as EdVerifyingKey};
use signature::{Signer, Verifier};
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
/// - Curve25519 (Classical ECDH)
/// - ML-KEM-768 (Post-Quantum KEM)
/// - ML-DSA-65 (Post-Quantum Signature, Primary Identity)
/// - Ed25519 (Classical Signature)
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
    /// The derived Ed25519 signing key.
    pub ed_private: EdSigningKey,
    /// The derived Ed25519 verifying key.
    pub ed_public: EdVerifyingKey,
}

impl core::fmt::Debug for OrbitalKeyPair {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("OrbitalKeyPair")
            .field("curve_public", &self.curve_public)
            .field("kem_public", &self.kem_public)
            .field("dsa_public", &self.dsa_public)
            .field("ed_public", &self.ed_public)
            .field("private_material", &"<REDACTED>")
            .finish()
    }
}

impl OrbitalKeyPair {
    /// Derive a Hybrid key pair from an existing orbital state.
    pub fn from_bodies(bodies: &[OrbitalBody], step: u64, g: kelvin_core::Fixed, softening: kelvin_core::Fixed) -> Self {
        // 1. Derive Curve25519 (Classical ECDH)
        let seed_ecc = extract_seed(bodies, step, g, softening, b"kelvin-curve25519-v1");
        let scalar = Scalar::from_bytes_mod_order_wide(&seed_ecc);
        let curve_private = StaticSecret::from(scalar.to_bytes());
        let curve_public = PublicKey::from(&curve_private);

        // 2. Derive ML-KEM-768 (Post-Quantum KEM)
        let seed_kem = extract_seed(bodies, step, g, softening, b"kelvin-ml-kem-v1");
        let kem_private = DecapsulationKey::<MlKem768>::from_seed(seed_kem.into());
        let kem_public = kem_private.encapsulation_key().clone();

        // 3. Derive ML-DSA-65 (Post-Quantum Signature)
        let seed_dsa = extract_seed(bodies, step, g, softening, b"kelvin-ml-dsa-v1");
        let dsa_seed_32: [u8; 32] = seed_dsa[0..32].try_into().expect("SHA3-512 must be 64 bytes");
        let dsa_private = SigningKey::<MlDsa65>::from_seed(&dsa_seed_32.into());
        let dsa_public = dsa_private.verifying_key().clone();

        // 4. Derive Ed25519 (Classical Signature)
        let seed_ed = extract_seed(bodies, step, g, softening, b"kelvin-ed25519-v1");
        let mut ed_seed = [0u8; 32];
        ed_seed.copy_from_slice(&seed_ed[..32]);
        let ed_private = EdSigningKey::from_bytes(&ed_seed);
        let ed_public = ed_private.verifying_key();

        OrbitalKeyPair {
            curve_private,
            curve_public,
            kem_private,
            kem_public,
            dsa_private,
            dsa_public,
            ed_private,
            ed_public,
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

        Ok(Self::from_bodies(&bodies, config.total_steps, config.g, config.softening))
    }

    /// Sign a message using ML-DSA-65 (Post-Quantum).
    ///
    /// Uses `ml-dsa`'s re-exported `signature` v3 `Signer` trait.
    pub fn sign_ml_dsa(&self, msg: &[u8]) -> Vec<u8> {
        use ml_dsa::signature::Signer;
        self.dsa_private
            .try_sign(msg)
            .expect("ML-DSA signing should not fail")
            .encode()
            .to_vec()
    }

    /// Verify an ML-DSA-65 signature.
    ///
    /// Uses `ml-dsa`'s re-exported `signature` v3 `Verifier` trait.
    pub fn verify_ml_dsa(&self, msg: &[u8], signature: &[u8]) -> Result<(), signature::Error> {
        use ml_dsa::signature::Verifier;
        let sig = ml_dsa::Signature::try_from(signature)
            .map_err(|_| signature::Error::new())?;
        self.dsa_public
            .verify(msg, &sig)
            .map_err(|_| signature::Error::new())
    }

    /// Sign a message using Ed25519 (Classical).
    pub fn sign_ed25519(&self, msg: &[u8]) -> ed25519_dalek::Signature {
        use signature::Signer;
        self.ed_private.sign(msg)
    }

    /// Verify an Ed25519 signature.
    pub fn verify_ed25519(&self, msg: &[u8], signature: &ed25519_dalek::Signature) -> Result<(), signature::Error> {
        use signature::Verifier;
        self.ed_public.verify(msg, signature)
    }
}

// ── RustCrypto `signature` trait adapters ──

/// Adapter implementing RustCrypto's `Signer` trait for ML-DSA-65.
///
/// Uses `ml-dsa`'s re-exported `signature` v3 internally.
pub struct MlDsaSigner(pub SigningKey<MlDsa65>);

impl signature::Signer<Vec<u8>> for MlDsaSigner {
    fn try_sign(&self, msg: &[u8]) -> Result<Vec<u8>, signature::Error> {
        use ml_dsa::signature::Signer as _;
        self.0
            .try_sign(msg)
            .map(|sig| sig.encode().to_vec())
            .map_err(|_| signature::Error::new())
    }
}

/// Adapter implementing RustCrypto's `Verifier` trait for ML-DSA-65.
///
/// Uses `ml-dsa`'s re-exported `signature` v3 internally.
pub struct MlDsaVerifier(pub VerifyingKey<MlDsa65>);

impl signature::Verifier<Vec<u8>> for MlDsaVerifier {
    fn verify(&self, msg: &[u8], signature: &Vec<u8>) -> Result<(), signature::Error> {
        use ml_dsa::signature::Verifier as _;
        let sig = ml_dsa::Signature::try_from(signature.as_slice())
            .map_err(|_| signature::Error::new())?;
        self.0
            .verify(msg, &sig)
            .map_err(|_| signature::Error::new())
    }
}

/// Adapter implementing RustCrypto's `Signer` trait for Ed25519.
pub struct Ed25519Signer(pub EdSigningKey);

impl signature::Signer<ed25519_dalek::Signature> for Ed25519Signer {
    fn try_sign(&self, msg: &[u8]) -> Result<ed25519_dalek::Signature, signature::Error> {
        Ok(self.0.sign(msg))
    }
}

/// Adapter implementing RustCrypto's `Verifier` trait for Ed25519.
pub struct Ed25519Verifier(pub EdVerifyingKey);

impl signature::Verifier<ed25519_dalek::Signature> for Ed25519Verifier {
    fn verify(&self, msg: &[u8], signature: &ed25519_dalek::Signature) -> Result<(), signature::Error> {
        self.0.verify(msg, signature)
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

        // Verify Ed25519 keys are deterministic
        assert_eq!(kp1.ed_public.to_bytes(), kp2.ed_public.to_bytes());
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
        assert_ne!(kp1.ed_public.to_bytes(), kp2.ed_public.to_bytes());
    }

    #[test]
    fn test_ml_dsa_sign_verify() {
        let config = test_config();
        let kp = OrbitalKeyPair::derive(&config).unwrap();
        let msg = b"Test message for ML-DSA signing";

        let signature = kp.sign_ml_dsa(msg);
        assert!(kp.verify_ml_dsa(msg, &signature).is_ok());

        // Tampered message should fail
        assert!(kp.verify_ml_dsa(b"wrong message", &signature).is_err());
    }

    #[test]
    fn test_ed25519_sign_verify() {
        let config = test_config();
        let kp = OrbitalKeyPair::derive(&config).unwrap();
        let msg = b"Test message for Ed25519 signing";

        let signature = kp.sign_ed25519(msg);
        assert!(kp.verify_ed25519(msg, &signature).is_ok());

        // Tampered message should fail
        assert!(kp.verify_ed25519(b"wrong message", &signature).is_err());
    }

    #[test]
    fn test_signature_trait_ml_dsa() {
        let config = test_config();
        let kp = OrbitalKeyPair::derive(&config).unwrap();
        let msg = b"Test for signature trait";

        let signer = MlDsaSigner(kp.dsa_private.clone());
        let verifier = MlDsaVerifier(kp.dsa_public.clone());

        let sig = signature::Signer::<Vec<u8>>::sign(&signer, msg);
        assert!(signature::Verifier::<Vec<u8>>::verify(&verifier, msg, &sig).is_ok());
    }

    #[test]
    fn test_signature_trait_ed25519() {
        let config = test_config();
        let kp = OrbitalKeyPair::derive(&config).unwrap();
        let msg = b"Test for Ed25519 signature trait";

        let signer = Ed25519Signer(kp.ed_private);
        let verifier = Ed25519Verifier(kp.ed_public);

        let sig = signature::Signer::<ed25519_dalek::Signature>::sign(&signer, msg);
        assert!(signature::Verifier::<ed25519_dalek::Signature>::verify(&verifier, msg, &sig).is_ok());
    }
}
