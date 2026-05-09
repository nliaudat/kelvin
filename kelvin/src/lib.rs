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
pub use kelvin_core::{Fixed, Vec3, OrbitalBody};
pub use kelvin_kdf::{OrbitalConfig, KeySchedule, ScheduleState, extract_seed};
pub use kelvin_stream::{ChaChaStream, StreamCipher};

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
pub struct Kelvin {
    config: OrbitalConfig,
    bodies: Vec<OrbitalBody>,
    schedule: KeySchedule,
    stream: ChaChaStream,
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
        );
        let result = lyapunov.estimate(1000, config.total_steps)?;

        if config.total_steps > result.safe_steps {
            return Err(KelvinError::InsufficientLyapunovTime {
                requested: config.total_steps,
                safe: result.safe_steps,
            });
        }

        // Clone bodies for simulation
        let mut bodies = config.bodies.clone();

        // Run initial simulation
        simulate(&mut bodies, config.total_steps, config.dt, config.softening);

        // Extract initial seed
        let seed = extract_seed(&bodies, config.total_steps, b"kelvin-orbital-state-v1");

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

        // Create stream cipher
        let stream = ChaChaStream::new(key, nonce);

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
        let planet = OrbitalBody::new(
            Fixed::from_raw(1 << 54), // ~1e-6 solar masses
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        OrbitalConfig::new(
            vec![sun, planet],
            50,  // Use fewer steps to stay within Lyapunov time
            10,
            Fixed::from_raw(1 << 44), // ~1e-6
            Fixed::from_raw(1 << 44), // ~1e-6
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
