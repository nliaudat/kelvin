//! # Kelvin — Orbital Chaos KDF Cryptosystem
//!
//! Top-level orchestrator for the Kelvin cryptosystem.
//!
//! Provides:
//! - `Kelvin` struct — main encryption/decryption entry point
//! - `KelvinError` — error types
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.** This is an experimental
//! cryptosystem that has not undergone formal cryptanalysis.
//!
//! ## Related Work
//!
//! - CryptoChaos (Harvard University, 2025): A hybrid chaos-based
//!   cryptographic framework combining deterministic chaos with X25519
//!   Diffie-Hellman key exchange and SHA3-256 hashing. Demonstrates
//!   academic interest in chaos-based cryptography for post-quantum
//!   applications.
//!
//! ## Example
//!
//! ```rust,ignore
//! use kelvin::{Kelvin, OrbitalConfig};
//!
//! let config = OrbitalConfig::from_json(json_str)?;
//! let mut k = Kelvin::new(config)?;
//! let mut data = b"Hello, world!".to_vec();
//! k.encrypt(&mut data)?;
//! k.decrypt(&mut data)?;
//! assert_eq!(&data, b"Hello, world!");
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod error;
mod encrypt;
mod decrypt;

pub use error::KelvinError;
pub use kelvin_core::{Fixed, Vec3, OrbitalBody, DEFAULT_G};
pub use kelvin_kdf::{OrbitalConfig, KeySchedule, ScheduleState, extract_seed, extract_seed_extended, OrbitalKeyPair, AsymmetricError};
pub use kelvin_stream::{ChaChaStream, StreamCipher};

#[cfg(feature = "aes-ni")]
pub use kelvin_stream::AesCtrStream;

use kelvin_core::simulate;
use kelvin_kdf::LyapunovEstimator;

/// Main entry point for the Kelvin cryptosystem.
///
/// Orchestrates the full pipeline:
/// 1. Validate OrbitalConfig
/// 2. Estimate Lyapunov time
/// 3. Run orbital simulation
/// 4. Extract seeds via SHA3-512
/// 5. Generate keystream via ChaCha20
#[derive(Debug)]
pub struct Kelvin {
    #[allow(dead_code)]
    config: OrbitalConfig,
    #[allow(dead_code)]
    bodies: Vec<OrbitalBody>,
    schedule: KeySchedule,
    stream: Box<dyn StreamCipher>,
    bytes_processed: u64,
}

impl Kelvin {
    /// Create a new Kelvin instance from a validated configuration.
    ///
    /// This runs the Lyapunov time estimator and initial orbital simulation.
    /// Setup time depends on the security level (seconds to minutes).
    pub fn new(config: OrbitalConfig) -> Result<Self, KelvinError> {
        // Validate config
        config.validate()?;

        // Estimate Lyapunov time
        let lyapunov = LyapunovEstimator::new(
            &config.bodies,
            config.dt,
            config.softening,
            config.g,
        );
        let result = lyapunov.estimate(1000, config.total_steps)?;

        if config.total_steps < result.safe_steps {
            return Err(KelvinError::InsufficientChaos {
                requested: config.total_steps,
                horizon: result.safe_steps,
            });
        }

        // Clone bodies for simulation
        let mut bodies = config.bodies.clone();

        // Run initial simulation
        simulate(&mut bodies, config.total_steps, config.dt, config.softening, config.g);

        // Extract initial 320-byte seed (5× SHA3-512 for enhanced entropy)
        let seed_vec = extract_seed_extended(&bodies, config.total_steps, b"kelvin-orbital-state-v1", 320);
        let mut seed = [0u8; 320];
        seed.copy_from_slice(&seed_vec);

        // Create key schedule
        let mut schedule = KeySchedule::new(
            seed,
            config.total_steps,
            config.reseed_interval,
            result.safe_steps,
        );

        // Get first key
        let (key, nonce) = schedule.next_key()
            .ok_or(KelvinError::SeedExhausted)?;

        // Create stream cipher (ChaChaStream takes a 12-byte nonce)
        let mut chacha_nonce = [0u8; 12];
        chacha_nonce.copy_from_slice(&nonce[..12]);
        let stream = Box::new(ChaChaStream::new(key, chacha_nonce));

        Ok(Kelvin {
            config,
            bodies,
            schedule,
            stream,
            bytes_processed: 0,
        })
    }

    /// Create a new Kelvin instance using the hardware-accelerated AES-256-CTR fallback.
    ///
    /// Available when the `aes-ni` feature is enabled.
    #[cfg(feature = "aes-ni")]
    pub fn new_aes(config: OrbitalConfig) -> Result<Self, KelvinError> {
        // Validate config
        config.validate()?;

        // Estimate Lyapunov time
        let lyapunov = LyapunovEstimator::new(
            &config.bodies,
            config.dt,
            config.softening,
            config.g,
        );
        let result = lyapunov.estimate(1000, config.total_steps)?;

        if config.total_steps < result.safe_steps {
            return Err(KelvinError::InsufficientChaos {
                requested: config.total_steps,
                horizon: result.safe_steps,
            });
        }

        // Clone bodies for simulation
        let mut bodies = config.bodies.clone();

        // Run initial simulation
        simulate(&mut bodies, config.total_steps, config.dt, config.softening, config.g);

        // Extract initial 320-byte seed (5× SHA3-512 for enhanced entropy)
        let seed_vec = extract_seed_extended(&bodies, config.total_steps, b"kelvin-orbital-state-v1", 320);
        let mut seed = [0u8; 320];
        seed.copy_from_slice(&seed_vec);

        // Create key schedule
        let mut schedule = KeySchedule::new(
            seed,
            config.total_steps,
            config.reseed_interval,
            result.safe_steps,
        );

        // Get first key
        let (key, nonce) = schedule.next_key()
            .ok_or(KelvinError::SeedExhausted)?;

        // Create stream cipher (AesCtrStream takes a 16-byte nonce)
        let stream = Box::new(AesCtrStream::new(key, nonce));

        Ok(Kelvin {
            config,
            bodies,
            schedule,
            stream,
            bytes_processed: 0,
        })
    }

    /// Encrypt data in-place (XOR with keystream).
    ///
    /// Encryption and decryption are identical operations (XOR).
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.stream.xor_in_place(data);
        self.bytes_processed += data.len() as u64;
        Ok(())
    }

    /// Decrypt data in-place (XOR with keystream).
    ///
    /// Identical to `encrypt` — XOR is its own inverse.
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.encrypt(data)
    }

    /// Total bytes processed (encrypted or decrypted) since initialization.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// Remaining safe bytes before orbital time exhaustion.
    pub fn remaining_safe_bytes(&self) -> u64 {
        self.schedule.remaining_bytes()
    }

    /// Derive the asymmetric Curve25519 key pair associated with this Kelvin instance.
    ///
    /// This utilizes the already simulated orbital state and does not require re-running the simulation.
    pub fn asymmetric_keypair(&self) -> OrbitalKeyPair {
        OrbitalKeyPair::from_bodies(&self.bodies, self.config.total_steps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::Fixed;

    fn test_config() -> OrbitalConfig {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
        let planet1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54), // ~1e-6 solar masses
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
            200,  // Use enough steps to exceed Lyapunov horizon
            10,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44), // ~1e-6
            kelvin_core::DEFAULT_G,
        ).unwrap()
    }

    #[test]
    fn test_round_trip_small() {
        let config = test_config();
        let mut k = Kelvin::new(config.clone()).unwrap();
        let mut data = vec![0xABu8; 64];
        let original = data.clone();
        k.encrypt(&mut data).unwrap();
        assert_ne!(data, original);
        // Create a new Kelvin instance for decryption (same config = same keystream)
        let mut k2 = Kelvin::new(config).unwrap();
        k2.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }
}
