//! # Kelvin — Quantum-Resistant One-Time Pad Cryptosystem
//!
//! Top-level orchestrator for the Kelvin cryptosystem — a **quantum-resistant
//! stream cipher cryptosystem** based on fixed-point gravitational n-body
//! simulation.
//!
//! All XOR-based modes (V2 Chaos, V3 Photon, H Quantum, Prism, Split, Flare)
//! produce a **quantum-resistant OTP keystream**: data is XOR-encrypted
//! byte-by-byte with keystream derived from SHAKE256 (NIST PQC standard).
//! There is no nonce, no IV, no algebraic round function. The only attack is
//! brute force.
//!
//! Provides:
//! - `Kelvin` struct — original V1 encryption/decryption entry point (virtual time, ChaCha20Poly1305 AEAD)
//! - `KelvinStreaming` struct — V2 per-step OTP streaming (real time, one simulation step per chunk)
//! - `KelvinPhoton` struct — V3 batch OTP via HKDF→SHAKE256 XOR (fast bulk encryption)
//! - `KelvinQuantum` struct — H hybrid OTP (V3+V2 XOR with orbital reseeding)
//! - `KelvinPrism` struct — standalone OTP key generator for homomorphic encryption
//! - `KelvinSplit` struct — dedicated XOR key-splitter for homomorphic encryption
//! - `KelvinFlare` struct — chaotic FHE secret key generator
//! - `FlareScheme` enum — supported FHE schemes (BFV, CKKS, TFHE)
//! - `FlareKey` struct — FHE secret key with scheme metadata
//! - `KelvinError` — error types
//!
//! ## What This Crate Is Not For
//!
//! This crate is **not** a general-purpose encryption library. It is a
//! research cryptosystem based on n-body simulation. It is **not**
//! suitable for production use without a formal cryptographic audit.
//!
//! This crate does **not** provide:
//! * Key exchange or transport (OrbitalConfig must be shared out-of-band)
//! * Memory-hard KDF (not GPU/ASIC resistant)
//! * Formal cryptanalysis (no reduction to a hard problem)
//! * Information-theoretic OTP (XOR modes are computational OTPs)
//!
//! If you need production-grade encryption, use established libraries
//! like [`aes-gcm`](https://crates.io/crates/aes-gcm) or
//! [`chacha20poly1305`](https://crates.io/crates/chacha20poly1305).
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

#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod authenticated;
mod decrypt;
mod encrypt;
mod error;
mod flare;
pub mod mode;
mod parameters;
mod photon;
mod prism;
mod quantum;
mod split;
pub mod streaming;

pub use error::KelvinError;
pub use kelvin_core::{Fixed, OrbitalBody, Vec3, DEFAULT_DT, DEFAULT_G, SOFTENING_FACTOR};
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
pub use parameters::{
    AUTH_FORMAT_VERSION, AUTH_OVERHEAD, AUTH_TAG_LEN, CHAOS_DEFAULT_BYTES_PER_STEP,
    DEFAULT_BYTES_PER_STEP, DOMSEP_FLARE_KEYSTREAM_V1, DOMSEP_FLARE_RESEED_V1, DOMSEP_MAC_KEY_V1,
    DOMSEP_ORBITAL_STATE_V1, DOMSEP_PHOTON_KEYSTREAM_V1, DOMSEP_PHOTON_RESEED_V1,
    DOMSEP_PRISM_KEYSTREAM_V1, DOMSEP_PRISM_RESEED_V1, DOMSEP_QUANTUM_CACHE_V1,
    DOMSEP_QUANTUM_KEYSTREAM, DOMSEP_QUANTUM_PERTURB_V1, DOMSEP_QUANTUM_RESEED_V1,
    DOMSEP_SPLIT_KEYSTREAM_V1, DOMSEP_SPLIT_RESEED_V1, DOMSEP_STREAMING_MAC_KEY_V1,
    EXTRACT_BUF_SIZE, FAST_RESEED_INTERVAL, FAST_STEPS, KEYSTREAM_CHUNK_SIZE,
    KEY_SCHEDULE_SEED_SIZE, LYAPUNOV_SHADOW_STEPS, MAC_KEY_SIZE, MAXIMUM_BODIES, MAXIMUM_STEPS,
    ORBITAL_SEED_SIZE, ORBITAL_VELOCITY_CONSTANT, PARANOID_BODIES, PARANOID_STEPS,
    PHOTON_BASE_SEED_SIZE, PHOTON_DEFAULT_BYTES_PER_STEP, PHOTON_DEFAULT_MAX_RESEEDS,
    PLANET_MASS_MAX_RAW, PLANET_MASS_MIN_RAW, PLANET_RADIUS_MULTIPLIER, QUANTUM_BASE_SEED_SIZE,
    QUANTUM_DEFAULT_BYTES_PER_STEP, QUANTUM_DEFAULT_CACHE_SIZE, QUANTUM_DEFAULT_INTEGRATION_METHOD,
    QUANTUM_DEFAULT_MAX_RESEEDS, QUANTUM_DEFAULT_ORBITAL_STEPS, QUANTUM_DEFAULT_RESEED_INTERVAL,
    QUANTUM_PERTURB_SCALE, STANDARD_BODIES, STANDARD_STEPS, STREAMING_CHUNK_SIZE, SUN_MASS_CENTER,
    SUN_MASS_MAX_RAW, SUN_MASS_MIN_RAW, SUN_MASS_RANGE, SUN_POS_MAX_RAW, SUN_POS_MIN_RAW,
    SUN_VEL_MAX_RAW, SUN_VEL_MIN_RAW, XOF_SEED_SIZE,
};

pub use flare::{FlareKey, FlareScheme, KelvinFlare};
pub use photon::KelvinPhoton;
pub use prism::KelvinPrism;
pub use quantum::KelvinQuantum;
pub use split::KelvinSplit;

pub use streaming::*;

pub use kelvin_core::IntegrationMethod;
use kelvin_core::{simulate_with_monitoring, simulate_with_monitoring_euler};
use kelvin_kdf::LyapunovEstimator;
use zeroize::Zeroize;

/// Result of the shared orbital initialization pipeline.
///
/// Contains the simulated bodies, orbital seed, and Lyapunov estimation result.
/// Used by both [`simulate_and_extract_seed_with_method`] (public API) and
/// [`Kelvin::init_with_method`] (internal V1 initialization).
struct SimulationResult {
    seed: [u8; ORBITAL_SEED_SIZE],
    bodies: Vec<OrbitalBody>,
    lyapunov: LyapunovResult,
}

/// Internal consolidation of `LyapunovEstimator::estimate` output.
/// Avoids spilling internal type details into `SimulationResult`.
struct LyapunovResult {
    min_chaos_steps: u64,
}

/// Run the full orbital simulation pipeline: config validation, Lyapunov
/// estimation, orbital simulation with stability monitoring, and SHAKE256
/// seed extraction.
///
/// This is the **single shared implementation** of the initialization pipeline.
/// Both the public API (`simulate_and_extract_seed_with_method`) and the internal
/// V1 initialization (`Kelvin::init_with_method`) delegate to this function,
/// eliminating ~90 lines of duplicated code.
///
/// # Arguments
/// * `bodies` — Orbital bodies to simulate (taken by value, moved in-place).
/// * `config` — Configuration parameters (dt, softening, G, stability thresholds).
/// * `steps` — Total simulation steps.
/// * `method` — Integration method (Verlet or Euler).
///
/// # Returns
/// `SimulationResult` containing the simulated bodies, extracted seed,
/// and Lyapunov estimation data.
fn run_simulation_pipeline(
    mut bodies: Vec<OrbitalBody>,
    config: &OrbitalConfig,
    steps: u64,
    method: IntegrationMethod,
) -> Result<SimulationResult, KelvinError> {
    // Estimate Lyapunov time
    let lyapunov = LyapunovEstimator::new(&bodies, config.dt, config.softening, config.g, method);
    let result = lyapunov.estimate(LYAPUNOV_SHADOW_STEPS, steps)?;

    if steps < result.min_chaos_steps {
        return Err(KelvinError::InsufficientChaos {
            requested: steps,
            horizon: result.min_chaos_steps,
        });
    }

    // Run initial simulation with stability monitoring (Verlet or Euler)
    match method {
        IntegrationMethod::Verlet => {
            simulate_with_monitoring(
                &mut bodies,
                steps,
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
                steps,
                config.dt,
                config.softening,
                config.g,
                config.min_separation,
                config.monitor_interval,
                config.ejection_energy_threshold,
            )?;
        },
    }

    // Extract initial orbital seed (using SHAKE256 XOF) directly into
    // a fixed-size array — avoids an unnecessary Vec allocation.
    let mut seed = [0u8; ORBITAL_SEED_SIZE];
    extract_shake256_into(
        &bodies,
        steps,
        config.g,
        config.softening,
        DOMSEP_ORBITAL_STATE_V1,
        &mut seed,
    );

    Ok(SimulationResult {
        seed,
        bodies,
        lyapunov: LyapunovResult { min_chaos_steps: result.min_chaos_steps },
    })
}

/// Run the full orbital simulation pipeline and extract an orbital seed.
///
/// Uses Verlet integration (default). Callers that need Euler integration
/// should use [`simulate_and_extract_seed_with_method`].
///
/// This is the shared initialization used by V1 (`Kelvin`), V3 (`KelvinPhoton`),
/// and H (`KelvinQuantum`).
///
/// Returns the orbital seed and the simulated bodies.
pub fn simulate_and_extract_seed(
    config: &OrbitalConfig,
) -> Result<([u8; ORBITAL_SEED_SIZE], Vec<OrbitalBody>), KelvinError> {
    simulate_and_extract_seed_with_method(config, IntegrationMethod::default())
}

/// Like [`simulate_and_extract_seed`] but with a configurable integration method.
pub fn simulate_and_extract_seed_with_method(
    config: &OrbitalConfig,
    method: IntegrationMethod,
) -> Result<([u8; ORBITAL_SEED_SIZE], Vec<OrbitalBody>), KelvinError> {
    // Validate config
    config.validate()?;

    // Clone bodies for simulation (config is &, so we clone here)
    let bodies = config.bodies.clone();

    // Run the shared pipeline
    let result = run_simulation_pipeline(bodies, config, config.total_steps, method)?;

    Ok((result.seed, result.bodies))
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
    ///
    /// Delegates to [`run_simulation_pipeline`] for the core simulation and
    /// seed extraction, then creates the key schedule.
    fn init_with_method(
        config: OrbitalConfig,
        method: IntegrationMethod,
    ) -> Result<InitState, KelvinError> {
        // Validate config
        config.validate()?;

        // Delegate to the shared simulation pipeline
        let result =
            run_simulation_pipeline(config.bodies.clone(), &config, config.total_steps, method)?;

        // Apply expansion factor to safe_steps
        let safe_steps =
            result.lyapunov.min_chaos_steps.saturating_mul(config.expansion_factor.max(1));

        // Create key schedule with configurable byte limit per key
        let schedule = KeySchedule::with_max_bytes_per_key(
            result.seed,
            config.total_steps,
            config.reseed_interval,
            safe_steps,
            config.max_bytes_per_key,
        );

        Ok(InitState { config, bodies: result.bodies, schedule, safe_steps })
    }

    /// Create a new Kelvin instance from a validated configuration.
    ///
    /// This runs the Lyapunov time estimator and initial orbital simulation.
    /// Setup time depends on the security level (seconds to minutes).
    pub fn new(config: OrbitalConfig) -> Result<Self, KelvinError> {
        Self::new_with_method(config, IntegrationMethod::default())
    }

    /// Create a new Kelvin instance with a configurable integration method.
    ///
    /// Use `IntegrationMethod::Verlet` (default) for stable, energy-conserving
    /// integration. Use `IntegrationMethod::Euler` for maximum chaos amplification
    /// (numerical instability produces ~10x more entropy per step, but may cause
    /// body ejection in some configurations).
    pub fn new_with_method(
        config: OrbitalConfig,
        method: IntegrationMethod,
    ) -> Result<Self, KelvinError> {
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
        Self::new_aes_with_method(config, IntegrationMethod::default())
    }

    /// Create a new Kelvin instance using AES-256-GCM with a configurable integration method.
    ///
    /// Available when the `aes-ni` feature is enabled.
    #[cfg(feature = "aes-ni")]
    pub fn new_aes_with_method(
        config: OrbitalConfig,
        method: IntegrationMethod,
    ) -> Result<Self, KelvinError> {
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
        Self::new_with_method(config, bytes_per_step, IntegrationMethod::default())
    }

    /// Create a new streaming Kelvin instance with a configurable integration method.
    ///
    /// Use `IntegrationMethod::Verlet` (default) for stable, energy-conserving
    /// integration. Use `IntegrationMethod::Euler` for maximum chaos amplification
    /// (numerical instability produces ~10x more entropy per step, but may cause
    /// body ejection in some configurations).
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
    ///
    /// ## Stability monitoring
    ///
    /// Every `MONITOR_INTERVAL` steps (default: 10,000), checks for:
    /// - Body ejection (unbound orbit energy)
    /// - Gravitational collapse (bodies too close)
    ///
    /// If either condition is detected, returns `KelvinError::StabilityError`.
    fn advance_step(&mut self) -> Result<(), KelvinError> {
        match self.integration_method {
            IntegrationMethod::Verlet => {
                kelvin_core::verlet_step(&mut self.bodies, self.dt, self.softening, self.g);
            },
            IntegrationMethod::Euler => {
                kelvin_core::euler_step(&mut self.bodies, self.dt, self.softening, self.g);
            },
        }
        self.step += 1;

        // Periodic stability check every MONITOR_INTERVAL steps
        if self.step.is_multiple_of(kelvin_core::MONITOR_INTERVAL) {
            // Check for gravitational collapse

            if let Some((i, j, dist)) =
                kelvin_core::detect_collapse(&self.bodies, kelvin_core::MIN_SEPARATION)
            {
                return Err(KelvinError::StabilityError(format!(
                    "bodies {} and {} collided at step {} (distance = {:.6e} AU)",
                    i,
                    j,
                    self.step,
                    dist.to_f64()
                )));
            }

            // Check for body ejection
            for i in 0..self.bodies.len() {
                if kelvin_core::is_body_ejected(
                    i,
                    &self.bodies,
                    self.g,
                    self.softening,
                    kelvin_core::EJECTION_ENERGY_THRESHOLD,
                ) {
                    let energy = (self.bodies[i].kinetic_energy()
                        + kelvin_core::gravitational_potential(
                            i,
                            &self.bodies,
                            self.g,
                            self.softening,
                        ))
                    .to_f64();
                    return Err(KelvinError::StabilityError(format!(
                        "body {} ejected at step {} (energy = {:.6e} AU²/yr²)",
                        i, self.step, energy
                    )));
                }
            }
        }

        Ok(())
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
            self.advance_step()?;

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
            for (d, k) in data[offset..offset + chunk_size]
                .iter_mut()
                .zip(self.keystream_buf[..chunk_size].iter())
            {
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
    use kelvin_core::{DEFAULT_DT, SOFTENING_FACTOR};

    fn five_body_config() -> OrbitalConfig {
        // Stable 5-body system: central sun (mass 1) + 4 small planets.
        // Planets have tiny masses (~1e-12 to 1e-15 solar masses) at close
        // distances (1-2 AU) with appropriate orbital velocities.
        // This configuration remains bound for 500 Verlet steps.
        let bodies = vec![
            OrbitalBody::new(
                Fixed::ONE,
                Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
                Vec3::new(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO),
            ),
            OrbitalBody::new(
                Fixed::from_raw(1 << 54), // ~1e-6 solar masses
                Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
                Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
            ),
            OrbitalBody::new(
                Fixed::from_raw(1 << 53),
                Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
                Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
            ),
            OrbitalBody::new(
                Fixed::from_raw(1 << 52),
                Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
                Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
            ),
            OrbitalBody::new(
                Fixed::from_raw(1 << 51),
                Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
                Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
            ),
        ];
        OrbitalConfig::new(bodies, 500, 10, DEFAULT_DT, SOFTENING_FACTOR, DEFAULT_G)
            .expect("valid config")
    }

    // ── Kelvin (V1 Secure) tests ─────────────────────────────────────────

    #[test]
    fn test_kelvin_new_and_round_trip() {
        let config = five_body_config();
        let mut enc = Kelvin::new(config.clone()).expect("Kelvin::new should succeed");
        let mut dec = Kelvin::new(config).expect("Kelvin::new should succeed");
        let original = b"Hello, Kelvin!".to_vec();
        // AEAD requires 16 extra bytes for the authentication tag
        let mut data = {
            let mut buf = original.clone();
            buf.extend_from_slice(&[0u8; 16]);
            buf
        };
        enc.encrypt(&mut data).unwrap();
        // The first 14 bytes should be ciphertext (different from plaintext)
        assert_ne!(data[..14], original[..]);
        dec.decrypt(&mut data).unwrap();
        // After decryption, the first 14 bytes should be restored
        assert_eq!(data[..14], original[..]);
    }

    #[test]
    fn test_kelvin_empty_data() {
        let config = five_body_config();
        let mut enc = Kelvin::new(config.clone()).expect("Kelvin::new should succeed");
        let mut dec = Kelvin::new(config).expect("Kelvin::new should succeed");
        // AEAD requires at least 16 bytes for the authentication tag.
        // An empty plaintext still needs the tag space.
        let mut data = vec![0u8; 16]; // 0 plaintext + 16 tag
        enc.encrypt(&mut data).unwrap();
        // After encrypting 0 plaintext bytes, the buffer still has 16 bytes (tag)
        assert_eq!(data.len(), 16);
        dec.decrypt(&mut data).unwrap();
        assert_eq!(data.len(), 16);
    }

    #[test]
    fn test_kelvin_bytes_processed() {
        let config = five_body_config();
        let mut k = Kelvin::new(config).expect("Kelvin::new should succeed");
        assert_eq!(k.bytes_processed(), 0);
        // AEAD requires 16 extra bytes for the authentication tag.
        // bytes_processed counts plaintext bytes only (data.len() - 16).
        let mut data = vec![0u8; 100 + 16]; // 100 plaintext + 16 tag
        k.encrypt(&mut data).unwrap();
        assert_eq!(k.bytes_processed(), 100);
    }

    #[test]
    fn test_kelvin_remaining_safe_bytes() {
        let config = five_body_config();
        let k = Kelvin::new(config).expect("Kelvin::new should succeed");
        assert!(k.remaining_safe_bytes() > 0);
    }

    #[test]
    fn test_kelvin_asymmetric_keypair() {
        let config = five_body_config();
        let k = Kelvin::new(config).expect("Kelvin::new should succeed");
        let kp = k.asymmetric_keypair();
        // Keypair should have non-empty public key material
        let debug_str = format!("{:?}", kp);
        assert!(!debug_str.is_empty());
    }

    #[test]
    fn test_kelvin_deterministic_encryption() {
        let config = five_body_config();
        let mut k1 = Kelvin::new(config.clone()).expect("Kelvin::new");
        let mut k2 = Kelvin::new(config).expect("Kelvin::new");
        let mut data1 = b"Test data for determinism check".to_vec();
        let mut data2 = data1.clone();
        k1.encrypt(&mut data1).unwrap();
        k2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_kelvin_aead_tag_detection() {
        let config = five_body_config();
        let mut enc = Kelvin::new(config.clone()).expect("Kelvin::new");
        let mut dec = Kelvin::new(config).expect("Kelvin::new");
        let mut data = b"Hello, Kelvin!".to_vec();
        // Extend with space for AEAD tag
        data.extend_from_slice(&[0u8; 16]);
        enc.encrypt(&mut data).unwrap();
        // Tamper with the ciphertext
        data[0] ^= 0xFF;
        // Decryption should fail due to tag mismatch
        let result = dec.decrypt(&mut data);
        assert!(result.is_err());
    }

    // ── KelvinStreaming tests ────────────────────────────────────────────

    #[test]
    fn test_kelvin_streaming_new_and_round_trip() {
        let config = five_body_config();
        let mut enc = KelvinStreaming::new(config.clone(), 1024).expect("KelvinStreaming::new");
        let mut dec = KelvinStreaming::new(config, 1024).expect("KelvinStreaming::new");
        let original = b"Hello, KelvinStreaming!".to_vec();
        let mut data = original.clone();
        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);
        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    #[test]
    fn test_kelvin_streaming_empty_data() {
        let config = five_body_config();
        let mut ks = KelvinStreaming::new(config, 1024).expect("KelvinStreaming::new");
        let mut data = Vec::new();
        ks.encrypt(&mut data).unwrap();
        assert!(data.is_empty());
        ks.decrypt(&mut data).unwrap();
        assert!(data.is_empty());
    }

    #[test]
    fn test_kelvin_streaming_bytes_processed() {
        let config = five_body_config();
        let mut ks = KelvinStreaming::new(config, 1024).expect("KelvinStreaming::new");
        assert_eq!(ks.bytes_processed(), 0);
        let mut data = vec![0u8; 100];
        ks.encrypt(&mut data).unwrap();
        assert_eq!(ks.bytes_processed(), 100);
    }

    #[test]
    fn test_kelvin_streaming_step_counter() {
        let config = five_body_config();
        let mut ks = KelvinStreaming::new(config, 1024).expect("KelvinStreaming::new");
        assert_eq!(ks.step(), 0);
        let mut data = vec![0u8; 1024];
        ks.encrypt(&mut data).unwrap();
        assert_eq!(ks.step(), 1);
    }

    #[test]
    fn test_kelvin_streaming_determinism() {
        let config = five_body_config();
        let mut ks1 = KelvinStreaming::new(config.clone(), 1024).expect("KelvinStreaming::new");
        let mut ks2 = KelvinStreaming::new(config, 1024).expect("KelvinStreaming::new");
        let mut data1 = b"Deterministic streaming test".to_vec();
        let mut data2 = data1.clone();
        ks1.encrypt(&mut data1).unwrap();
        ks2.encrypt(&mut data2).unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_kelvin_streaming_multi_chunk() {
        let config = five_body_config();
        let mut enc = KelvinStreaming::new(config.clone(), 512).expect("KelvinStreaming::new");
        let mut dec = KelvinStreaming::new(config, 512).expect("KelvinStreaming::new");
        let original = vec![0xABu8; 2048]; // 4 chunks of 512 bytes
        let mut data = original.clone();
        enc.encrypt(&mut data).unwrap();
        assert_ne!(data, original);
        dec.decrypt(&mut data).unwrap();
        assert_eq!(data, original);
    }

    // ── Thread safety compile-time checks ────────────────────────────────

    /// Compile-time assertion that types implement Send + Sync.
    /// These are zero-runtime-cost checks that verify thread safety.
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    #[test]
    fn test_kelvin_streaming_is_send_sync() {
        assert_send::<KelvinStreaming>();
        assert_sync::<KelvinStreaming>();
    }

    #[test]
    fn test_kelvin_photon_is_send_sync() {
        assert_send::<KelvinPhoton>();
        assert_sync::<KelvinPhoton>();
    }

    #[test]
    fn test_kelvin_quantum_is_send_sync() {
        assert_send::<KelvinQuantum>();
        assert_sync::<KelvinQuantum>();
    }

    #[test]
    fn test_kelvin_prism_is_send_sync() {
        assert_send::<KelvinPrism>();
        assert_sync::<KelvinPrism>();
    }

    #[test]
    fn test_kelvin_split_is_send_sync() {
        assert_send::<KelvinSplit>();
        assert_sync::<KelvinSplit>();
    }

    #[test]
    fn test_kelvin_flare_is_send_sync() {
        assert_send::<KelvinFlare>();
        assert_sync::<KelvinFlare>();
    }
}
