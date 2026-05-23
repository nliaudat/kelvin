//! # Kelvin — Orbital Chaos KDF Cryptosystem
//!
//! Top-level orchestrator for the Kelvin cryptosystem.
//!
//! Provides:
//! - `Kelvin` struct — original V1 encryption/decryption entry point (virtual time)
//! - `KelvinStreaming` struct — V2 streaming encryption/decryption (real time, one step per chunk)
//! - `KelvinError` — error types
//!
//! ## V2 Streaming Mode
//!
//! `KelvinStreaming` replaces the virtual-time key schedule with a true
//! per-step simulation. Each chunk of data consumes one simulation step:
//!
//! ```rust,ignore
//! use kelvin::{KelvinStreaming, OrbitalConfig};
//!
//! let config = OrbitalConfig::from_json(json_str)?;
//! let mut ks = KelvinStreaming::new(config, 1024 * 1024)?; // 1 MiB per step
//! let mut data = b"Hello, world!".to_vec();
//! ks.encrypt(&mut data)?;
//! ks.decrypt(&mut data)?;
//! assert_eq!(&data, b"Hello, world!");
//! ```
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
//! ## Example (V1)
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

mod authenticated;
mod decrypt;
mod encrypt;
mod error;
mod photon;
mod quantum;

pub use error::KelvinError;
pub use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_G};
pub use kelvin_kdf::{
    extract_seed, extract_shake256, extract_shake256_into, AsymmetricError, KeySchedule,
    OrbitalConfig, OrbitalKeyPair, OrbitalState, ScheduleState,
};
pub use kelvin_stream::{ChaChaStream, StreamCipher};

#[cfg(feature = "aes-ni")]
pub use kelvin_stream::AesGcmStream;

pub use authenticated::{
    KelvinPhotonAuthenticated, KelvinQuantumAuthenticated, KelvinStreamingAuthenticated,
};
pub use photon::KelvinPhoton;
pub use quantum::{KelvinQuantum, DEFAULT_CACHE_SIZE, DEFAULT_RESEED_INTERVAL, DEFAULT_ORBITAL_STEPS};

use kelvin_core::{simulate_with_monitoring, simulate_with_monitoring_euler};
use kelvin_kdf::LyapunovEstimator;
use zeroize::Zeroize;

/// Integration method for the n-body gravitational simulation.
///
/// - **Verlet** (default): Symplectic Velocity Verlet. Energy-conserving,
///   time-reversible. Used by default for backward compatibility.
/// - **Euler**: Explicit Euler integration. Numerical instability amplifies
///   chaos ~10x faster than Verlet, producing more entropy per step.
///   Use `--euler` to opt in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrationMethod {
    /// Symplectic Velocity Verlet (default, backward compatible).
    Verlet,
    /// Explicit Euler (maximum chaos amplification).
    Euler,
}

impl Default for IntegrationMethod {
    fn default() -> Self {
        IntegrationMethod::Verlet
    }
}

/// Run the full orbital simulation pipeline and extract a 2048-byte seed.
///
/// This is the shared initialization used by V1 (`Kelvin`), V3 (`KelvinPhoton`),
/// and H (`KelvinQuantum`). It performs:
/// 1. Config validation
/// 2. Lyapunov time estimation
/// 3. Full orbital simulation with stability monitoring
/// 4. SHAKE256 seed extraction
///
/// Returns the 2048-byte seed and the simulated bodies.
pub fn simulate_and_extract_seed(config: &OrbitalConfig) -> Result<([u8; 2048], Vec<OrbitalBody>), KelvinError> {
    simulate_and_extract_seed_with_method(config, IntegrationMethod::Verlet)
}

/// Like [`simulate_and_extract_seed`] but with a configurable integration method.
pub fn simulate_and_extract_seed_with_method(
    config: &OrbitalConfig,
    method: IntegrationMethod,
) -> Result<([u8; 2048], Vec<OrbitalBody>), KelvinError> {

    // Validate config
    config.validate()?;

    // Estimate Lyapunov time
    let lyapunov =
        LyapunovEstimator::new(&config.bodies, config.dt, config.softening, config.g);
    let result = lyapunov.estimate(1000, config.total_steps)?;

    if config.total_steps < result.min_chaos_steps {
        return Err(KelvinError::InsufficientChaos {
            requested: config.total_steps,
            horizon: result.min_chaos_steps,
        });
    }

    // Clone bodies for simulation
    let mut bodies = config.bodies.clone();

    // Run initial simulation with stability monitoring (Verlet or Euler)
    match method {
        IntegrationMethod::Verlet => {
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
        },
        IntegrationMethod::Euler => {
            simulate_with_monitoring_euler(
                &mut bodies,
                config.total_steps,
                config.dt,
                config.softening,
                config.g,
                config.min_separation,
                config.monitor_interval,
                config.ejection_energy_threshold,
            )?;
        },
    }

    // Extract initial 2048-byte seed (using SHAKE256 XOF) directly into
    // a fixed-size array — avoids an unnecessary Vec allocation.
    let mut seed = [0u8; 2048];
    extract_shake256_into(
        &bodies,
        config.total_steps,
        config.g,
        config.softening,
        b"kelvin-orbital-state-v1",
        &mut seed,
    );

    Ok((seed, bodies))
}

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
    fn init_with_method(config: OrbitalConfig, method: IntegrationMethod) -> Result<InitState, KelvinError> {
        // Validate config
        config.validate()?;

        // Estimate Lyapunov time
        let lyapunov =
            LyapunovEstimator::new(&config.bodies, config.dt, config.softening, config.g);
        let result = lyapunov.estimate(1000, config.total_steps)?;

        if config.total_steps < result.min_chaos_steps {
            return Err(KelvinError::InsufficientChaos {
                requested: config.total_steps,
                horizon: result.min_chaos_steps,
            });
        }

        // Clone bodies for simulation
        let mut bodies = config.bodies.clone();

        // Run initial simulation with stability monitoring (Verlet or Euler)
        match method {
            IntegrationMethod::Verlet => {
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
            },
            IntegrationMethod::Euler => {
                simulate_with_monitoring_euler(
                    &mut bodies,
                    config.total_steps,
                    config.dt,
                    config.softening,
                    config.g,
                    config.min_separation,
                    config.monitor_interval,
                    config.ejection_energy_threshold,
                )?;
            },
        }

        // Extract initial 2048-byte seed (using SHAKE256 XOF) directly into
        // a fixed-size array — avoids an unnecessary Vec allocation.
        let mut seed = [0u8; 2048];
        extract_shake256_into(
            &bodies,
            config.total_steps,
            config.g,
            config.softening,
            b"kelvin-orbital-state-v1",
            &mut seed,
        );

        // Apply expansion factor to safe_steps
        let safe_steps = result.min_chaos_steps.saturating_mul(config.expansion_factor.max(1));

        // Create key schedule with configurable byte limit per key
        let schedule = KeySchedule::with_max_bytes_per_key(
            seed,
            config.total_steps,
            config.reseed_interval,
            safe_steps,
            config.max_bytes_per_key,
        );

        Ok(InitState { config, bodies, schedule, safe_steps })
    }

    /// Create a new Kelvin instance from a validated configuration.
    ///
    /// This runs the Lyapunov time estimator and initial orbital simulation.
    /// Setup time depends on the security level (seconds to minutes).
    pub fn new(config: OrbitalConfig) -> Result<Self, KelvinError> {
        Self::new_with_method(config, IntegrationMethod::Verlet)
    }

    /// Create a new Kelvin instance with a configurable integration method.
    ///
    /// Use `IntegrationMethod::Euler` for maximum chaos amplification
    /// (numerical instability produces ~10x more entropy per step).
    /// Use `IntegrationMethod::Verlet` (default) for backward compatibility.
    pub fn new_with_method(config: OrbitalConfig, method: IntegrationMethod) -> Result<Self, KelvinError> {
        let mut state = Self::init_with_method(config, method)?;

        // Get first key
        let (key, nonce) = state.schedule.next_key().ok_or(KelvinError::SeedExhausted)?;

        // Create stream cipher with configurable byte limit
        let stream =
            Box::new(ChaChaStream::with_max_bytes(key, nonce, state.config.max_bytes_per_key));

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
        Self::new_aes_with_method(config, IntegrationMethod::Verlet)
    }

    /// Create a new Kelvin instance using AES-256-GCM with a configurable integration method.
    ///
    /// Available when the `aes-ni` feature is enabled.
    #[cfg(feature = "aes-ni")]
    pub fn new_aes_with_method(config: OrbitalConfig, method: IntegrationMethod) -> Result<Self, KelvinError> {
        let mut state = Self::init_with_method(config, method)?;

        // Get first key
        let (key, nonce) = state.schedule.next_key().ok_or(KelvinError::SeedExhausted)?;

        // Create AES-256-GCM stream cipher with configurable byte limit
        let stream =
            Box::new(AesGcmStream::with_max_bytes(key, nonce, state.config.max_bytes_per_key));

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
        OrbitalKeyPair::from_bodies(
            &self.bodies,
            self.config.total_steps,
            self.config.g,
            self.config.softening,
        )
    }

    /// Rotate the stream cipher key by deriving the next key from the schedule.
    ///
    /// Called automatically when the current key approaches its maximum safe
    /// byte limit. This prevents nonce reuse and provides forward secrecy.
    fn rotate_key(&mut self) -> Result<(), KelvinError> {
        let (key, nonce) = self.schedule.next_key().ok_or(KelvinError::SeedExhausted)?;
        // Rekey the existing stream in-place to preserve the cipher variant
        // (ChaCha20Poly1305 vs AES-256-GCM) chosen at construction time.
        self.stream.rekey(key, nonce);
        Ok(())
    }
}

impl Drop for Kelvin {
    fn drop(&mut self) {
        // Zeroize the stream cipher key material
        self.stream.zeroize_key_material();
        // Zeroize the bodies (simulated orbital state)
        self.bodies.zeroize();
        // Zeroize bytes_processed counter
        self.bytes_processed.zeroize();
        // config and schedule are zeroized by their own Drop impls
    }
}

// ============================================================================
// V2 Streaming Mode — True One-Time Pad with Per-Step Simulation
// ============================================================================

/// V2 streaming cryptosystem: one simulation step per data chunk.
///
/// Unlike V1 (`Kelvin`) which runs the entire simulation upfront and then
/// uses a virtual key schedule, `KelvinStreaming` advances the simulation
/// by one step for each chunk of data processed. This means:
///
/// - **Unlimited keystream**: keep simulating as long as you need
/// - **True OTP**: each step's chaotic state is unique and unpredictable
/// - **Predicted ETA**: file_size / bytes_per_step = steps needed, benchmark gives steps/sec
///
/// ## How it works
///
/// For each `process_chunk()` call:
/// 1. Advance simulation by one Verlet step
/// 2. Extract keystream from current orbital state via SHAKE256 XOF
/// 3. XOR the data with the keystream
///
/// ## Example
///
/// ```rust,ignore
/// use kelvin::{KelvinStreaming, OrbitalConfig};
///
/// let config = OrbitalConfig::from_json(json_str)?;
/// let mut ks = KelvinStreaming::new(config, 1024 * 1024)?; // 1 MiB per step
/// let mut data = b"Hello, world!".to_vec();
/// ks.encrypt(&mut data)?;
/// ks.decrypt(&mut data)?;
/// assert_eq!(&data, b"Hello, world!");
/// ```
#[derive(Debug)]
pub struct KelvinStreaming {
    /// Current orbital bodies (simulation state).
    bodies: Vec<OrbitalBody>,
    /// Current step counter.
    step: u64,
    /// Simulation parameters.
    dt: Fixed,
    softening: Fixed,
    g: Fixed,
    /// Number of keystream bytes produced per step.
    bytes_per_step: u64,
    /// Total bytes processed so far.
    bytes_processed: u64,
    /// Domain separator for SHAKE256 extraction.
    domain_separator: [u8; 32],
    /// Reusable keystream buffer to avoid repeated allocations.
    keystream_buf: Vec<u8>,
    /// Integration method (Verlet or Euler).
    integration_method: IntegrationMethod,
}

impl KelvinStreaming {
    /// Create a new streaming Kelvin instance.
    ///
    /// `config` is the shared orbital configuration.
    /// `bytes_per_step` is how many keystream bytes each simulation step produces
    /// (e.g., 1 MiB = 1,048,576). Larger values mean fewer steps for a given file size.
    ///
    /// This does NOT run the full simulation upfront — it only validates the config
    /// and initializes the body state. The simulation advances one step per chunk.
    pub fn new(config: OrbitalConfig, bytes_per_step: u64) -> Result<Self, KelvinError> {
        Self::new_with_method(config, bytes_per_step, IntegrationMethod::Verlet)
    }

    /// Create a new streaming Kelvin instance with a configurable integration method.
    ///
    /// Use `IntegrationMethod::Euler` for maximum chaos amplification
    /// (numerical instability produces ~10x more entropy per step).
    /// Use `IntegrationMethod::Verlet` (default) for backward compatibility.
    pub fn new_with_method(
        config: OrbitalConfig,
        bytes_per_step: u64,
        method: IntegrationMethod,
    ) -> Result<Self, KelvinError> {
        // Validate config
        config.validate()?;

        // Clone bodies (initial state, no simulation yet)
        let bodies = config.bodies.clone();

        let domain_separator = *b"kelvin-streaming-v2-v1-000000000";

        let bps = bytes_per_step.max(1) as usize;

        Ok(KelvinStreaming {
            bodies,
            step: 0,
            dt: config.dt,
            softening: config.softening,
            g: config.g,
            bytes_per_step: bps as u64,
            bytes_processed: 0,
            domain_separator,
            keystream_buf: vec![0u8; bps],
            integration_method: method,
        })
    }

    /// Advance the simulation by one step using the configured integration method.
    fn advance_step(&mut self) {
        match self.integration_method {
            IntegrationMethod::Verlet => {
                kelvin_core::verlet_step(&mut self.bodies, self.dt, self.softening, self.g);
            },
            IntegrationMethod::Euler => {
                kelvin_core::euler_step(&mut self.bodies, self.dt, self.softening, self.g);
            },
        }
        self.step += 1;
    }

    /// Process a chunk of data: advance simulation, extract keystream, XOR.
    ///
    /// This is the core operation. It processes the input in fixed-size chunks
    /// of `bytes_per_step` bytes, advancing the simulation by one step
    /// for each chunk. This ensures the keystream is deterministic regardless
    /// of how the caller chunks the data, as long as call sizes are multiples
    /// of `bytes_per_step`.
    ///
    /// For each chunk:
    /// 1. Advances the n-body simulation by one step (Verlet or Euler)
    /// 2. Extracts `bytes_per_step` bytes of keystream from the current state
    /// 3. XORs the chunk with the keystream
    ///
    /// A final partial chunk still advances the simulation by one step, but
    /// only XORs the needed bytes.
    ///
    /// The same operation encrypts and decrypts (XOR is its own inverse).
    pub fn process_chunk(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        if data.is_empty() {
            return Ok(());
        }

        let bps = self.bytes_per_step as usize;
        let mut offset = 0;

        while offset < data.len() {
            let remaining = data.len() - offset;
            let chunk_size = std::cmp::min(remaining, bps);

            // 1. Advance simulation by one step (Verlet or Euler)
            self.advance_step();

            // 2. Extract keystream into the reusable buffer via SHAKE256 XOF
            //    This avoids allocating a new Vec<u8> for every chunk iteration.
            extract_shake256_into(
                &self.bodies,
                self.step,
                self.g,
                self.softening,
                &self.domain_separator,
                &mut self.keystream_buf,
            );

            // 3. XOR chunk with keystream
            for (d, k) in data[offset..offset + chunk_size].iter_mut().zip(self.keystream_buf[..chunk_size].iter()) {
                *d ^= k;
            }

            offset += chunk_size;
        }

        self.bytes_processed += data.len() as u64;

        Ok(())
    }


    /// Encrypt data in-place (same as process_chunk).
    pub fn encrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.process_chunk(data)
    }

    /// Decrypt data in-place (same as process_chunk, XOR is its own inverse).
    pub fn decrypt(&mut self, data: &mut [u8]) -> Result<(), KelvinError> {
        self.process_chunk(data)
    }

    /// Get the current step counter.
    pub fn step(&self) -> u64 {
        self.step
    }

    /// Get the total bytes processed so far.
    pub fn bytes_processed(&self) -> u64 {
        self.bytes_processed
    }

    /// Get the number of keystream bytes produced per step.
    pub fn bytes_per_step(&self) -> u64 {
        self.bytes_per_step
    }

    /// Benchmark the simulation speed on this hardware.
    ///
    /// Runs `sample_steps` steps using the configured integration method
    /// and returns the rate in steps/second.
    /// Use this to estimate ETA for a given file size.
    pub fn benchmark(&self, sample_steps: u64) -> f64 {
        let mut bodies = self.bodies.clone();
        let start = std::time::Instant::now();
        for _ in 0..sample_steps {
            match self.integration_method {
                IntegrationMethod::Verlet => {
                    kelvin_core::verlet_step(&mut bodies, self.dt, self.softening, self.g);
                },
                IntegrationMethod::Euler => {
                    kelvin_core::euler_step(&mut bodies, self.dt, self.softening, self.g);
                },
            }
        }
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            sample_steps as f64 / elapsed
        } else {
            f64::MAX
        }
    }

    /// Estimate the time needed to process a file of `file_size` bytes.
    ///
    /// Returns (steps_needed, estimated_seconds).
    pub fn estimate_time(&self, file_size: u64, steps_per_sec: f64) -> (u64, f64) {
        let steps_needed = file_size.div_ceil(self.bytes_per_step);
        let estimated_secs =
            if steps_per_sec > 0.0 { steps_needed as f64 / steps_per_sec } else { f64::MAX };
        (steps_needed, estimated_secs)
    }
}

impl Drop for KelvinStreaming {
    fn drop(&mut self) {
        self.bodies.zeroize();
        self.step.zeroize();
        self.bytes_processed.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::Fixed;

    fn test_config() -> OrbitalConfig {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
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
            200, // Use enough steps to exceed Lyapunov horizon
            10,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44), // ~1e-6
            kelvin_core::DEFAULT_G,
        )
        .unwrap()
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

    // ─── V2 Streaming Tests ────────────────────────────────────────────────

    fn streaming_config() -> OrbitalConfig {
        // Use a simple 5-body config (minimum required by validation)
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
            1000,
            100,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        )
        .unwrap()
    }

    #[test]
    fn test_streaming_round_trip() {
        let config = streaming_config();
        let mut ks = KelvinStreaming::new(config, 64).unwrap();
        let mut data = b"Hello, Kelvin V2 streaming!".to_vec();
        let original = data.clone();

        ks.encrypt(&mut data).unwrap();
        assert_ne!(data, original, "encrypted data should differ from plaintext");

        // Decrypt with a new instance (same config = same keystream)
        let mut ks2 = KelvinStreaming::new(streaming_config(), 64).unwrap();
        ks2.decrypt(&mut data).unwrap();
        assert_eq!(data, original, "round-trip should restore original");
    }

    #[test]
    fn test_streaming_determinism() {
        let config = streaming_config();
        let mut ks1 = KelvinStreaming::new(config.clone(), 64).unwrap();
        let mut ks2 = KelvinStreaming::new(config, 64).unwrap();

        let mut data1 = b"Determinism test data".to_vec();
        let mut data2 = data1.clone();

        ks1.encrypt(&mut data1).unwrap();
        ks2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2, "two instances should produce identical ciphertext");
    }

    #[test]
    fn test_streaming_multi_chunk() {
        let config = streaming_config();
        let mut ks = KelvinStreaming::new(config, 32).unwrap();

        let chunk1 = b"First chunk of data!".to_vec();
        let chunk2 = b"Second chunk, different.".to_vec();
        let orig1 = chunk1.clone();
        let orig2 = chunk2.clone();

        let mut c1 = chunk1;
        let mut c2 = chunk2;

        ks.encrypt(&mut c1).unwrap();
        ks.encrypt(&mut c2).unwrap();

        assert_ne!(c1, orig1);
        assert_ne!(c2, orig2);

        // Decrypt with new instance
        let mut ks2 = KelvinStreaming::new(streaming_config(), 32).unwrap();
        ks2.decrypt(&mut c1).unwrap();
        ks2.decrypt(&mut c2).unwrap();
        assert_eq!(c1, orig1);
        assert_eq!(c2, orig2);
    }

    #[test]
    fn test_streaming_empty_data() {
        let config = streaming_config();
        let mut ks = KelvinStreaming::new(config, 64).unwrap();
        let mut empty: Vec<u8> = vec![];
        ks.encrypt(&mut empty).unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_streaming_step_counter() {
        let config = streaming_config();
        let mut ks = KelvinStreaming::new(config, 64).unwrap();
        assert_eq!(ks.step(), 0);

        let mut data = vec![0u8; 64];
        ks.encrypt(&mut data).unwrap();
        assert_eq!(ks.step(), 1);

        ks.encrypt(&mut data).unwrap();
        assert_eq!(ks.step(), 2);
    }

    #[test]
    fn test_streaming_bytes_processed() {
        let config = streaming_config();
        let mut ks = KelvinStreaming::new(config, 64).unwrap();
        assert_eq!(ks.bytes_processed(), 0);

        let mut data = vec![0u8; 100];
        ks.encrypt(&mut data).unwrap();
        assert_eq!(ks.bytes_processed(), 100);

        let mut data2 = vec![0u8; 50];
        ks.encrypt(&mut data2).unwrap();
        assert_eq!(ks.bytes_processed(), 150);
    }

    #[test]
    fn test_streaming_estimate_time() {
        let config = streaming_config();
        let ks = KelvinStreaming::new(config, 1024 * 1024).unwrap(); // 1 MiB per step

        let (steps, secs) = ks.estimate_time(10 * 1024 * 1024, 10000.0); // 10 MiB file
        assert_eq!(steps, 10); // 10 MiB / 1 MiB = 10 steps
        assert!((secs - 0.001).abs() < 0.001); // 10 / 10000 = 0.001s
    }

    #[test]
    fn test_streaming_benchmark() {
        let config = streaming_config();
        let ks = KelvinStreaming::new(config, 64).unwrap();
        let rate = ks.benchmark(100);
        assert!(rate > 0.0, "benchmark should return positive rate");
    }
}
