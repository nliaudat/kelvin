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
pub use kelvin_kdf::{OrbitalConfig, KeySchedule, ScheduleState, extract_seed, extract_shake256, OrbitalKeyPair, AsymmetricError};
pub use kelvin_stream::{ChaChaStream, StreamCipher};

#[cfg(feature = "aes-ni")]
pub use kelvin_stream::AesGcmStream;

use kelvin_core::simulate_with_monitoring;
use kelvin_kdf::LyapunovEstimator;

/// Main entry point for the Kelvin cryptosystem.
///
/// Orchestrates the full pipeline:
/// 1. Validate OrbitalConfig
/// 2. Estimate Lyapunov time
/// 3. Run orbital simulation
/// 4. Extract seeds via SHA3-512
/// 5. Generate keystream via ChaCha20Poly1305 AEAD
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

/// Shared state produced by the initialization pipeline.
struct InitState {
    config: OrbitalConfig,
    bodies: Vec<OrbitalBody>,
    schedule: KeySchedule,
    #[allow(dead_code)]
    safe_steps: u64,
}

impl Kelvin {
    /// Run the shared initialization pipeline (validation, Lyapunov estimation,
    /// simulation, seed extraction, key schedule creation).
    fn init(config: OrbitalConfig) -> Result<InitState, KelvinError> {
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

        // Run initial simulation with stability monitoring
        simulate_with_monitoring(
            &mut bodies,
            config.total_steps,
            config.dt,
            config.softening,
            config.g,
            config.min_separation,
            config.monitor_interval,
            config.ejection_energy_threshold,
        )?;

        // Extract initial 2048-byte seed (using SHAKE256 XOF)
        let seed_vec = extract_shake256(&bodies, config.total_steps, config.g, config.softening, b"kelvin-orbital-state-v1", 2048);
        let mut seed = [0u8; 2048];
        seed.copy_from_slice(&seed_vec);

        // Create key schedule
        let schedule = KeySchedule::new(
            seed,
            config.total_steps,
            config.reseed_interval,
            result.safe_steps,
        );

        Ok(InitState {
            config,
            bodies,
            schedule,
            safe_steps: result.safe_steps,
        })
    }

    /// Create a new Kelvin instance from a validated configuration.
    ///
    /// This runs the Lyapunov time estimator and initial orbital simulation.
    /// Setup time depends on the security level (seconds to minutes).
    pub fn new(config: OrbitalConfig) -> Result<Self, KelvinError> {
        let mut state = Self::init(config)?;

        // Get first key
        let (key, nonce) = state.schedule.next_key()
            .ok_or(KelvinError::SeedExhausted)?;

        // Create stream cipher (nonce is already [u8; 12])
        let stream = Box::new(ChaChaStream::new(key, nonce));

        Ok(Kelvin {
            config: state.config,
            bodies: state.bodies,
            schedule: state.schedule,
            stream,
            bytes_processed: 0,
        })
    }

    /// Create a new Kelvin instance using the hardware-accelerated AES-256-GCM fallback.
    ///
    /// Available when the `aes-ni` feature is enabled.
    #[cfg(feature = "aes-ni")]
    pub fn new_aes(config: OrbitalConfig) -> Result<Self, KelvinError> {
        let mut state = Self::init(config)?;

        // Get first key
        let (key, nonce) = state.schedule.next_key()
            .ok_or(KelvinError::SeedExhausted)?;

        // Create AES-256-GCM stream cipher (nonce is already [u8; 12])
        let stream = Box::new(AesGcmStream::new(key, nonce));

        Ok(Kelvin {
            config: state.config,
            bodies: state.bodies,
            schedule: state.schedule,
            stream,
            bytes_processed: 0,
        })
    }

    /// Encrypt data in-place using AEAD.
    ///
    /// The buffer must have 16 extra bytes after the plaintext for the
    /// Poly1305/GMAC authentication tag.
    ///
    /// This is a convenience wrapper around [`encrypt_in_place`](Self::encrypt_in_place).
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.encrypt_in_place(data)
    }

    /// Decrypt data in-place using AEAD.
    ///
    /// The buffer must contain ciphertext + 16-byte authentication tag.
    ///
    /// This is a convenience wrapper around [`decrypt_in_place`](Self::decrypt_in_place).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.decrypt_in_place(data)
    }

    /// Total bytes processed (encrypted or decrypted) since initialization.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// Remaining safe bytes before orbital time exhaustion.
    pub fn remaining_safe_bytes(&self) -> u64 {
        self.schedule.remaining_bytes()
    }

    /// Derive the hybrid post-quantum key pair associated with this Kelvin instance.
    ///
    /// This utilizes the already simulated orbital state and does not require
    /// re-running the simulation.
    pub fn asymmetric_keypair(&self) -> OrbitalKeyPair {
        OrbitalKeyPair::from_bodies(&self.bodies, self.config.total_steps, self.config.g, self.config.softening)
    }

    /// Rotate the stream cipher key by deriving the next key from the schedule.
    ///
    /// Called automatically when the current key approaches its maximum safe
    /// byte limit. This prevents nonce reuse and provides forward secrecy.
    fn rotate_key(&mut self) -> Result<(), KelvinError> {
        let (key, nonce) = self.schedule.next_key()
            .ok_or(KelvinError::SeedExhausted)?;
        // Rekey the existing stream in-place to preserve the cipher variant
        // (ChaCha20Poly1305 vs AES-256-GCM) chosen at construction time.
        self.stream.rekey(key, nonce);
        Ok(())
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
        // Buffer needs 16 extra bytes for AEAD tag
        let mut data = vec![0xABu8; 64 + 16];
        let original = data.clone();
        k.encrypt_in_place(&mut data).unwrap();
        // Ciphertext portion (first 64 bytes) should differ from plaintext
        assert_ne!(&data[..64], &original[..64]);
        // Create a new Kelvin instance for decryption (same config = same keystream)
        let mut k2 = Kelvin::new(config).unwrap();
        k2.decrypt_in_place(&mut data).unwrap();
        // Plaintext portion should be restored; tag portion is overwritten during decrypt
        assert_eq!(&data[..64], &original[..64]);
    }
}
