//! Orbital configuration — the shared secret format.
//!
//! The `OrbitalConfig` defines the initial conditions for the n-body
//! simulation. It is the shared secret between communicating parties.
//!
//! All non-trivial parameters are part of the config — there are no
//! hidden constants that could cause two parties to diverge.

use alloc::vec::Vec;

use kelvin_core::{Fixed, OrbitalBody, Vec3};
#[allow(unused_imports)]
use kelvin_core::{
    DEFAULT_DT, DEFAULT_G, DEFAULT_RESEED_INTERVAL, EJECTION_ENERGY_THRESHOLD, MAX_BODIES, MAX_DT,
    MIN_BODIES, MIN_DT, MIN_SEPARATION, MONITOR_INTERVAL, SOFTENING_FACTOR,
};
use zeroize::Zeroize;

/// Size of the extra binary fields beyond the core body + simulation data.
///
/// Layout: min_separation(16) + ejection_energy_threshold(16) + monitor_interval(8)
///         + min_bodies(4) + max_bodies(4) + min_dt(16) + max_dt(16)
///         + min_g(16) + max_g(16) + expansion_factor(8) + max_bytes_per_key(8)
///         = 128 bytes
const BINARY_EXTRA_SIZE: usize = 16 + 16 + 8 + 4 + 4 + 16 + 16 + 16 + 16 + 8 + 8;

/// Orbital configuration — the shared secret.
///
/// Contains:
/// - Initial positions, velocities, and masses of all bodies
/// - Simulation parameters (steps, dt, softening, G)
/// - Stability thresholds (min_separation, ejection_energy, monitor_interval)
/// - Validation bounds (min/max bodies, dt, G)
///
/// Every non-trivial parameter is part of the config so that two parties
/// with different compiled versions still derive identical keystreams.
#[derive(Clone, Debug)]
pub struct OrbitalConfig {
    /// Initial orbital bodies.
    pub bodies: Vec<OrbitalBody>,
    /// Total number of simulation steps.
    pub total_steps: u64,
    /// Steps between key schedule reseeds.
    pub reseed_interval: u64,
    /// Time step in years.
    pub dt: Fixed,
    /// Softening factor in AU.
    pub softening: Fixed,
    /// Gravitational constant G.
    pub g: Fixed,

    // ── Stability thresholds (part of the key) ──
    /// Minimum allowed separation between any two bodies (in AU).
    /// If any pair comes closer, the system is considered collapsed.
    /// Default: ~6e-8 AU ≈ 9 km.
    pub min_separation: Fixed,
    /// Energy threshold for detecting unbound (ejected) bodies.
    /// A body with total specific energy >= this threshold is ejected.
    /// Default: ~1.9e-6 (relative energy).
    pub ejection_energy_threshold: Fixed,
    /// Steps between stability checks during simulation.
    /// Default: 10,000 (1% of a standard simulation).
    pub monitor_interval: u64,

    // ── Validation bounds (part of the key) ──
    /// Minimum number of bodies required.
    /// Default: 5.
    pub min_bodies: usize,
    /// Maximum number of bodies supported.
    /// Default: 100.
    pub max_bodies: usize,
    /// Minimum allowed time step in years.
    /// Default: ~1e-8 yr (0.315 seconds).
    pub min_dt: Fixed,
    /// Maximum allowed time step in years.
    /// Default: ~1e-1 yr (36.5 days).
    pub max_dt: Fixed,
    /// Minimum allowed gravitational constant.
    /// Default: 1.0.
    pub min_g: Fixed,
    /// Maximum allowed gravitational constant.
    /// Default: 1000.0.
    pub max_g: Fixed,

    // ── Key schedule expansion ──
    /// Multiplier for the virtual step budget (safe_steps).
    ///
    /// Each key schedule reseed consumes `reseed_interval` virtual steps.
    /// The total virtual budget is `min_chaos_steps × expansion_factor`.
    /// Default: 1 (no expansion). Max: 10,000.
    ///
    /// Cryptographically safe: HKDF-SHA512 can derive millions of keys
    /// from a single 2048-byte seed. The expansion factor only affects
    /// the exhaustion limit, not the key derivation itself.
    pub expansion_factor: u64,
    /// Maximum safe bytes per key before automatic rotation.
    ///
    /// Default: 4 GiB (1 << 32). Max: 256 GiB (RFC 8439 ChaCha20 limit).
    /// Larger values reduce key rotation frequency at the cost of
    /// increased exposure if a key is compromised.
    pub max_bytes_per_key: u64,
}

impl OrbitalConfig {
    /// Create a new orbital configuration.
    ///
    /// Returns an error if validation fails.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        bodies: Vec<OrbitalBody>,
        total_steps: u64,
        reseed_interval: u64,
        dt: Fixed,
        softening: Fixed,
        g: Fixed,
    ) -> Result<Self, ConfigError> {
        // Use defaults for all stability/validation fields
        OrbitalConfig::new_full(
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
            MIN_SEPARATION,
            EJECTION_ENERGY_THRESHOLD,
            MONITOR_INTERVAL,
            MIN_BODIES,
            MAX_BODIES,
            MIN_DT,
            MAX_DT,
            Fixed::from_int(1),
            Fixed::from_int(1000),
        )
    }

    /// Create a new orbital configuration with all parameters specified.
    ///
    /// Every non-trivial parameter is explicit — no hidden constants.
    #[allow(clippy::too_many_arguments)]
    pub fn new_full(
        bodies: Vec<OrbitalBody>,
        total_steps: u64,
        reseed_interval: u64,
        dt: Fixed,
        softening: Fixed,
        g: Fixed,
        min_separation: Fixed,
        ejection_energy_threshold: Fixed,
        monitor_interval: u64,
        min_bodies: usize,
        max_bodies: usize,
        min_dt: Fixed,
        max_dt: Fixed,
        min_g: Fixed,
        max_g: Fixed,
    ) -> Result<Self, ConfigError> {
        OrbitalConfig::new_full_ext(
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
            min_separation,
            ejection_energy_threshold,
            monitor_interval,
            min_bodies,
            max_bodies,
            min_dt,
            max_dt,
            min_g,
            max_g,
            1,       // default expansion_factor
            1 << 32, // default max_bytes_per_key (4 GiB)
        )
    }

    /// Create a new orbital configuration with all parameters including
    /// key schedule expansion settings.
    ///
    /// `expansion_factor` multiplies the virtual step budget (default: 1, max: 10,000).
    /// `max_bytes_per_key` sets the safe byte limit per key (default: 4 GiB, max: 256 GiB).
    #[allow(clippy::too_many_arguments)]
    pub fn new_full_ext(
        bodies: Vec<OrbitalBody>,
        total_steps: u64,
        reseed_interval: u64,
        dt: Fixed,
        softening: Fixed,
        g: Fixed,
        min_separation: Fixed,
        ejection_energy_threshold: Fixed,
        monitor_interval: u64,
        min_bodies: usize,
        max_bodies: usize,
        min_dt: Fixed,
        max_dt: Fixed,
        min_g: Fixed,
        max_g: Fixed,
        expansion_factor: u64,
        max_bytes_per_key: u64,
    ) -> Result<Self, ConfigError> {
        let config = OrbitalConfig {
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
            min_separation,
            ejection_energy_threshold,
            monitor_interval,
            min_bodies,
            max_bodies,
            min_dt,
            max_dt,
            min_g,
            max_g,
            expansion_factor,
            max_bytes_per_key,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// Checks:
    /// - Number of bodies is within [min_bodies, max_bodies]
    /// - All masses are positive
    /// - dt is within [min_dt, max_dt]
    /// - softening is positive
    /// - total_steps > 0
    /// - reseed_interval > 0 and <= total_steps
    /// - G is within [min_g, max_g]
    /// - min_separation > 0
    /// - ejection_energy_threshold > 0
    /// - monitor_interval > 0
    /// - min_bodies >= 2 (need at least 2 for any dynamics)
    /// - max_bodies >= min_bodies
    /// - min_dt > 0, max_dt >= min_dt
    /// - min_g > 0, max_g >= min_g
    ///
    /// Bodyguard checks (reject bad systems at creation time):
    /// - No two bodies at identical positions (zero distance)
    /// - No body already on escape trajectory at step 0
    /// - No two bodies already closer than min_separation at step 0
    pub fn validate(&self) -> Result<(), ConfigError> {
        // ── Validation bound sanity ──
        if self.min_bodies < 2 {
            return Err(ConfigError::InvalidMinBodies(self.min_bodies));
        }
        if self.max_bodies < self.min_bodies {
            return Err(ConfigError::InvalidMaxBodies {
                max: self.max_bodies,
                min: self.min_bodies,
            });
        }
        if self.min_dt <= Fixed::ZERO {
            return Err(ConfigError::InvalidMinDt(self.min_dt));
        }
        if self.max_dt < self.min_dt {
            return Err(ConfigError::InvalidMaxDt { max: self.max_dt, min: self.min_dt });
        }
        if self.min_g <= Fixed::ZERO {
            return Err(ConfigError::InvalidMinG(self.min_g));
        }
        if self.max_g < self.min_g {
            return Err(ConfigError::InvalidMaxG { max: self.max_g, min: self.min_g });
        }

        // ── Body count ──
        if self.bodies.len() < self.min_bodies {
            return Err(ConfigError::TooFewBodies {
                count: self.bodies.len(),
                min: self.min_bodies,
            });
        }
        if self.bodies.len() > self.max_bodies {
            return Err(ConfigError::TooManyBodies {
                count: self.bodies.len(),
                max: self.max_bodies,
            });
        }

        // ── Masses ──
        for (i, body) in self.bodies.iter().enumerate() {
            if body.mass <= Fixed::ZERO {
                return Err(ConfigError::NonPositiveMass { body_index: i });
            }
        }

        // ── Simulation parameters ──
        if self.dt < self.min_dt || self.dt > self.max_dt {
            return Err(ConfigError::InvalidDt {
                dt: self.dt,
                min_dt: self.min_dt,
                max_dt: self.max_dt,
            });
        }
        if self.softening <= Fixed::ZERO {
            return Err(ConfigError::InvalidSoftening);
        }
        if self.total_steps == 0 {
            return Err(ConfigError::ZeroSteps);
        }
        if self.reseed_interval == 0 || self.reseed_interval > self.total_steps {
            return Err(ConfigError::InvalidReseedInterval);
        }
        if self.g < self.min_g || self.g > self.max_g {
            return Err(ConfigError::InvalidG { g: self.g, min_g: self.min_g, max_g: self.max_g });
        }

        // ── Stability thresholds ──
        if self.min_separation <= Fixed::ZERO {
            return Err(ConfigError::InvalidMinSeparation(self.min_separation));
        }
        if self.ejection_energy_threshold <= Fixed::ZERO {
            return Err(ConfigError::InvalidEjectionThreshold(self.ejection_energy_threshold));
        }
        if self.monitor_interval == 0 {
            return Err(ConfigError::InvalidMonitorInterval);
        }

        // ── Key schedule expansion ──
        if self.expansion_factor < 1 {
            return Err(ConfigError::InvalidExpansionFactor(self.expansion_factor));
        }
        if self.expansion_factor > 10_000 {
            return Err(ConfigError::InvalidExpansionFactor(self.expansion_factor));
        }
        if self.max_bytes_per_key < 1 {
            return Err(ConfigError::InvalidMaxBytesPerKey(self.max_bytes_per_key));
        }
        // Max 256 GiB (RFC 8439 ChaCha20 limit: 2^32 - 1 blocks × 64 bytes)
        let max_allowed_bytes: u64 = (1u64 << 32).saturating_sub(1) * 64;
        if self.max_bytes_per_key > max_allowed_bytes {
            return Err(ConfigError::InvalidMaxBytesPerKey(self.max_bytes_per_key));
        }

        // ── Bodyguard: reject bad systems at creation time ──

        // 1. Check for identical positions (zero distance)
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                let diff = self.bodies[j].position - self.bodies[i].position;
                if diff.length_squared() == Fixed::ZERO {
                    return Err(ConfigError::IdenticalPositions { body_i: i, body_j: j });
                }
            }
        }

        // 2. Check for initial collapse (bodies already too close)
        for i in 0..self.bodies.len() {
            for j in (i + 1)..self.bodies.len() {
                let diff = self.bodies[j].position - self.bodies[i].position;
                let dist = diff.length();
                if dist < self.min_separation {
                    return Err(ConfigError::InitialCollapse {
                        body_i: i,
                        body_j: j,
                        distance: dist,
                        min_separation: self.min_separation,
                    });
                }
            }
        }

        // 3. Check for initial ejection (body already on escape trajectory)
        for i in 0..self.bodies.len() {
            if kelvin_core::is_body_ejected(
                i,
                &self.bodies,
                self.g,
                self.softening,
                self.ejection_energy_threshold,
            ) {
                return Err(ConfigError::InitialEjection { body_index: i });
            }
        }

        Ok(())
    }

    /// Serialize to JSON string.
    #[cfg(feature = "serde")]
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string(self).map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON string.
    #[cfg(feature = "serde")]
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Format: [n_bodies: u32][body_data: n × 112 bytes][total_steps: u64]
    ///         [reseed_interval: u64][dt: i128][softening: i128][g: i128]
    ///         [min_separation: i128][ejection_energy_threshold: i128]
    ///         [monitor_interval: u64][min_bodies: u32][max_bodies: u32]
    ///         [min_dt: i128][max_dt: i128][min_g: i128][max_g: i128]
    ///         [expansion_factor: u64][max_bytes_per_key: u64]
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(
            4 + self.bodies.len() * 112 + 8 + 8 + 16 + 16 + 16 + BINARY_EXTRA_SIZE,
        );

        // Number of bodies (u32)
        buf.extend_from_slice(&(self.bodies.len() as u32).to_le_bytes());

        // Body data
        for body in &self.bodies {
            buf.extend_from_slice(&body.mass.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.position.x.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.position.y.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.position.z.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.velocity.x.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.velocity.y.to_raw().to_le_bytes());
            buf.extend_from_slice(&body.velocity.z.to_raw().to_le_bytes());
        }

        // Simulation parameters
        buf.extend_from_slice(&self.total_steps.to_le_bytes());
        buf.extend_from_slice(&self.reseed_interval.to_le_bytes());
        buf.extend_from_slice(&self.dt.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.softening.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.g.to_raw().to_le_bytes());

        // Stability thresholds
        buf.extend_from_slice(&self.min_separation.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.ejection_energy_threshold.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.monitor_interval.to_le_bytes());

        // Validation bounds
        buf.extend_from_slice(&(self.min_bodies as u32).to_le_bytes());
        buf.extend_from_slice(&(self.max_bodies as u32).to_le_bytes());
        buf.extend_from_slice(&self.min_dt.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.max_dt.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.min_g.to_raw().to_le_bytes());
        buf.extend_from_slice(&self.max_g.to_raw().to_le_bytes());

        // Key schedule expansion
        buf.extend_from_slice(&self.expansion_factor.to_le_bytes());
        buf.extend_from_slice(&self.max_bytes_per_key.to_le_bytes());

        buf
    }

    /// Deserialize from binary format.
    ///
    /// Returns an error if:
    /// - The data is too short for the declared header
    /// - Trailing bytes remain after parsing (rejects malformed data)
    /// - The parsed configuration fails validation
    pub fn from_binary(data: &[u8]) -> Result<Self, ConfigError> {
        let mut offset = 0;

        if data.len() < 4 {
            return Err(ConfigError::InvalidBinary("data too short".into()));
        }

        let n_bodies = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        let body_size = 112; // 7 × i128
        let header_size = 4 + n_bodies * body_size + 8 + 8 + 16 + 16 + 16 + BINARY_EXTRA_SIZE;

        if data.len() < header_size {
            return Err(ConfigError::InvalidBinary("data too short for bodies".into()));
        }

        let mut bodies = Vec::with_capacity(n_bodies);
        for _ in 0..n_bodies {
            let mass = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16]
                    .try_into()
                    .map_err(|_| ConfigError::InvalidBinary("body mass read failed".into()))?,
            ));
            offset += 16;

            let px = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body position x read failed".into())
                })?,
            ));
            offset += 16;
            let py = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body position y read failed".into())
                })?,
            ));
            offset += 16;
            let pz = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body position z read failed".into())
                })?,
            ));
            offset += 16;

            let vx = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body velocity x read failed".into())
                })?,
            ));
            offset += 16;
            let vy = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body velocity y read failed".into())
                })?,
            ));
            offset += 16;
            let vz = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().map_err(|_| {
                    ConfigError::InvalidBinary("body velocity z read failed".into())
                })?,
            ));
            offset += 16;

            bodies.push(OrbitalBody::new(mass, Vec3::new(px, py, pz), Vec3::new(vx, vy, vz)));
        }

        let total_steps = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("total_steps read failed".into()))?,
        );
        offset += 8;

        let reseed_interval = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("reseed_interval read failed".into()))?,
        );
        offset += 8;

        let dt = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("dt read failed".into()))?,
        ));
        offset += 16;

        let softening = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("softening read failed".into()))?,
        ));
        offset += 16;

        let g = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("g read failed".into()))?,
        ));
        offset += 16;

        // Stability thresholds
        let min_separation = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("min_separation read failed".into()))?,
        ));
        offset += 16;

        let ejection_energy_threshold =
            Fixed::from_raw(i128::from_le_bytes(data[offset..offset + 16].try_into().map_err(
                |_| ConfigError::InvalidBinary("ejection_energy_threshold read failed".into()),
            )?));
        offset += 16;

        let monitor_interval = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("monitor_interval read failed".into()))?,
        );
        offset += 8;

        // Validation bounds
        let min_bodies = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        let max_bodies = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]) as usize;
        offset += 4;

        let min_dt = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("min_dt read failed".into()))?,
        ));
        offset += 16;

        let max_dt = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("max_dt read failed".into()))?,
        ));
        offset += 16;

        let min_g = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("min_g read failed".into()))?,
        ));
        offset += 16;

        let max_g = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("max_g read failed".into()))?,
        ));
        offset += 16;

        // Key schedule expansion
        let expansion_factor = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("expansion_factor read failed".into()))?,
        );
        offset += 8;

        let max_bytes_per_key = u64::from_le_bytes(
            data[offset..offset + 8]
                .try_into()
                .map_err(|_| ConfigError::InvalidBinary("max_bytes_per_key read failed".into()))?,
        );
        offset += 8;

        // Check for trailing bytes — reject malformed data
        if offset != data.len() {
            return Err(ConfigError::InvalidBinary(format!(
                "trailing bytes: expected {} bytes, got {}",
                offset,
                data.len()
            )));
        }

        let config = OrbitalConfig {
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
            min_separation,
            ejection_energy_threshold,
            monitor_interval,
            min_bodies,
            max_bodies,
            min_dt,
            max_dt,
            min_g,
            max_g,
            expansion_factor,
            max_bytes_per_key,
        };
        config.validate()?;
        Ok(config)
    }
}

impl Drop for OrbitalConfig {
    fn drop(&mut self) {
        // Zeroize the bodies (mass, position, velocity) — the shared secret
        self.bodies.zeroize();
        // Zeroize simulation parameters
        self.total_steps.zeroize();
        self.reseed_interval.zeroize();
        self.dt.zeroize();
        self.softening.zeroize();
        self.g.zeroize();
        // Zeroize stability thresholds
        self.min_separation.zeroize();
        self.ejection_energy_threshold.zeroize();
        self.monitor_interval.zeroize();
        // Zeroize validation bounds
        self.min_bodies.zeroize();
        self.max_bodies.zeroize();
        self.min_dt.zeroize();
        self.max_dt.zeroize();
        self.min_g.zeroize();
        self.max_g.zeroize();
        // Zeroize key schedule expansion
        self.expansion_factor.zeroize();
        self.max_bytes_per_key.zeroize();
    }
}

/// Errors from configuration validation.
#[derive(Clone, Debug, thiserror::Error)]
pub enum ConfigError {
    /// Too few bodies.
    #[error("too few bodies: {count}, need at least {min}")]
    TooFewBodies {
        /// Number of bodies provided.
        count: usize,
        /// Minimum required.
        min: usize,
    },
    /// Too many bodies.
    #[error("too many bodies: {count}, max is {max}")]
    TooManyBodies {
        /// Number of bodies provided.
        count: usize,
        /// Maximum allowed.
        max: usize,
    },
    /// A body has non-positive mass.
    #[error("body {body_index} has non-positive mass")]
    NonPositiveMass {
        /// Index of the offending body.
        body_index: usize,
    },
    /// Time step is outside acceptable bounds.
    #[error("time step {dt} is outside acceptable range [{min_dt}, {max_dt}]")]
    InvalidDt {
        /// The provided time step.
        dt: Fixed,
        /// Minimum allowed time step.
        min_dt: Fixed,
        /// Maximum allowed time step.
        max_dt: Fixed,
    },
    /// Softening factor must be positive.
    #[error("softening factor must be positive")]
    InvalidSoftening,
    /// Total steps must be > 0.
    #[error("total steps must be > 0")]
    ZeroSteps,
    /// Reseed interval must be > 0 and <= total_steps.
    #[error("reseed interval must be > 0 and <= total_steps")]
    InvalidReseedInterval,
    /// Gravitational constant is outside bounds.
    #[error("gravitational constant {g} is outside range [{min_g}, {max_g}]")]
    InvalidG {
        /// The provided G.
        g: Fixed,
        /// Minimum allowed G.
        min_g: Fixed,
        /// Maximum allowed G.
        max_g: Fixed,
    },
    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Invalid binary data.
    #[error("invalid binary data: {0}")]
    InvalidBinary(String),

    // ── New validation bound errors ──
    /// Minimum bodies must be >= 2.
    #[error("min_bodies must be >= 2, got {0}")]
    InvalidMinBodies(usize),
    /// Maximum bodies must be >= minimum bodies.
    #[error("max_bodies {max} must be >= min_bodies {min}")]
    InvalidMaxBodies {
        /// Maximum bodies provided.
        max: usize,
        /// Minimum bodies.
        min: usize,
    },
    /// Minimum dt must be positive.
    #[error("min_dt must be positive, got {0}")]
    InvalidMinDt(Fixed),
    /// Maximum dt must be >= minimum dt.
    #[error("max_dt {max} must be >= min_dt {min}")]
    InvalidMaxDt {
        /// Maximum dt provided.
        max: Fixed,
        /// Minimum dt.
        min: Fixed,
    },
    /// Minimum G must be positive.
    #[error("min_g must be positive, got {0}")]
    InvalidMinG(Fixed),
    /// Maximum G must be >= minimum G.
    #[error("max_g {max} must be >= min_g {min}")]
    InvalidMaxG {
        /// Maximum G provided.
        max: Fixed,
        /// Minimum G.
        min: Fixed,
    },

    // ── Stability threshold errors ──
    /// Minimum separation must be positive.
    #[error("min_separation must be positive, got {0}")]
    InvalidMinSeparation(Fixed),
    /// Ejection energy threshold must be positive.
    #[error("ejection_energy_threshold must be positive, got {0}")]
    InvalidEjectionThreshold(Fixed),
    /// Monitor interval must be > 0.
    #[error("monitor_interval must be > 0")]
    InvalidMonitorInterval,

    // ── Key schedule expansion errors ──
    /// Expansion factor must be >= 1 and <= 10,000.
    #[error("expansion_factor must be >= 1 and <= 10,000, got {0}")]
    InvalidExpansionFactor(u64),
    /// Max bytes per key must be >= 1 and <= 256 GiB.
    #[error("max_bytes_per_key must be >= 1 and <= 256 GiB, got {0}")]
    InvalidMaxBytesPerKey(u64),

    // ── Bodyguard errors ──
    /// Two bodies have identical positions (zero distance).
    #[error("bodies {body_i} and {body_j} have identical positions")]
    IdenticalPositions {
        /// Index of the first body.
        body_i: usize,
        /// Index of the second body.
        body_j: usize,
    },
    /// Two bodies are already closer than min_separation at step 0.
    #[error("bodies {body_i} and {body_j} are too close: distance {distance} < min_separation {min_separation}")]
    InitialCollapse {
        /// Index of the first body.
        body_i: usize,
        /// Index of the second body.
        body_j: usize,
        /// Distance between the bodies.
        distance: Fixed,
        /// Minimum allowed separation.
        min_separation: Fixed,
    },
    /// A body is already on an escape trajectory at step 0.
    #[error("body {body_index} is already on an escape trajectory at step 0")]
    InitialEjection {
        /// Index of the ejected body.
        body_index: usize,
    },
}

#[cfg(feature = "serde")]
impl serde::Serialize for OrbitalConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OrbitalConfig", 17)?;

        state.serialize_field("bodies", &self.bodies)?;
        state.serialize_field("total_steps", &self.total_steps)?;
        state.serialize_field("reseed_interval", &self.reseed_interval)?;
        state.serialize_field("dt", &self.dt.to_raw())?;
        state.serialize_field("softening", &self.softening.to_raw())?;
        state.serialize_field("g", &self.g.to_raw())?;
        state.serialize_field("min_separation", &self.min_separation.to_raw())?;
        state.serialize_field(
            "ejection_energy_threshold",
            &self.ejection_energy_threshold.to_raw(),
        )?;
        state.serialize_field("monitor_interval", &self.monitor_interval)?;
        state.serialize_field("min_bodies", &self.min_bodies)?;
        state.serialize_field("max_bodies", &self.max_bodies)?;
        state.serialize_field("min_dt", &self.min_dt.to_raw())?;
        state.serialize_field("max_dt", &self.max_dt.to_raw())?;
        state.serialize_field("min_g", &self.min_g.to_raw())?;
        state.serialize_field("max_g", &self.max_g.to_raw())?;
        state.serialize_field("expansion_factor", &self.expansion_factor)?;
        state.serialize_field("max_bytes_per_key", &self.max_bytes_per_key)?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OrbitalConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use core::fmt;
        use serde::de::{self, MapAccess, Visitor};

        #[derive(Default)]
        struct ConfigFields {
            bodies: Option<Vec<OrbitalBody>>,
            total_steps: Option<u64>,
            reseed_interval: Option<u64>,
            dt_raw: Option<i128>,
            softening_raw: Option<i128>,
            g_raw: Option<i128>,
            min_separation_raw: Option<i128>,
            ejection_energy_threshold_raw: Option<i128>,
            monitor_interval: Option<u64>,
            min_bodies: Option<usize>,
            max_bodies: Option<usize>,
            min_dt_raw: Option<i128>,
            max_dt_raw: Option<i128>,
            min_g_raw: Option<i128>,
            max_g_raw: Option<i128>,
            expansion_factor: Option<u64>,
            max_bytes_per_key: Option<u64>,
        }

        struct ConfigVisitor;

        impl<'de> Visitor<'de> for ConfigVisitor {
            type Value = OrbitalConfig;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("struct OrbitalConfig")
            }

            fn visit_map<V: MapAccess<'de>>(self, mut map: V) -> Result<OrbitalConfig, V::Error> {
                let mut fields = ConfigFields::default();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "bodies" => fields.bodies = Some(map.next_value()?),
                        "total_steps" => fields.total_steps = Some(map.next_value()?),
                        "reseed_interval" => fields.reseed_interval = Some(map.next_value()?),
                        "dt" => fields.dt_raw = Some(map.next_value()?),
                        "softening" => fields.softening_raw = Some(map.next_value()?),
                        "g" => fields.g_raw = Some(map.next_value()?),
                        "min_separation" => fields.min_separation_raw = Some(map.next_value()?),
                        "ejection_energy_threshold" => {
                            fields.ejection_energy_threshold_raw = Some(map.next_value()?)
                        },
                        "monitor_interval" => fields.monitor_interval = Some(map.next_value()?),
                        "min_bodies" => fields.min_bodies = Some(map.next_value()?),
                        "max_bodies" => fields.max_bodies = Some(map.next_value()?),
                        "min_dt" => fields.min_dt_raw = Some(map.next_value()?),
                        "max_dt" => fields.max_dt_raw = Some(map.next_value()?),
                        "min_g" => fields.min_g_raw = Some(map.next_value()?),
                        "max_g" => fields.max_g_raw = Some(map.next_value()?),
                        "expansion_factor" => fields.expansion_factor = Some(map.next_value()?),
                        "max_bytes_per_key" => fields.max_bytes_per_key = Some(map.next_value()?),
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        },
                    }
                }

                let bodies = fields.bodies.ok_or_else(|| de::Error::missing_field("bodies"))?;
                let total_steps =
                    fields.total_steps.ok_or_else(|| de::Error::missing_field("total_steps"))?;
                let reseed_interval = fields.reseed_interval.unwrap_or(DEFAULT_RESEED_INTERVAL);
                let dt = Fixed::from_raw(fields.dt_raw.unwrap_or_else(|| DEFAULT_DT.to_raw()));
                let softening = Fixed::from_raw(
                    fields.softening_raw.unwrap_or_else(|| SOFTENING_FACTOR.to_raw()),
                );
                let g = Fixed::from_raw(fields.g_raw.unwrap_or_else(|| DEFAULT_G.to_raw()));

                // Stability thresholds (optional, use defaults)
                let min_separation = Fixed::from_raw(
                    fields.min_separation_raw.unwrap_or_else(|| MIN_SEPARATION.to_raw()),
                );
                let ejection_energy_threshold = Fixed::from_raw(
                    fields
                        .ejection_energy_threshold_raw
                        .unwrap_or_else(|| EJECTION_ENERGY_THRESHOLD.to_raw()),
                );
                let monitor_interval = fields.monitor_interval.unwrap_or(MONITOR_INTERVAL);

                // Validation bounds (optional, use defaults)
                let min_bodies = fields.min_bodies.unwrap_or(MIN_BODIES);
                let max_bodies = fields.max_bodies.unwrap_or(MAX_BODIES);
                let min_dt = Fixed::from_raw(fields.min_dt_raw.unwrap_or_else(|| MIN_DT.to_raw()));
                let max_dt = Fixed::from_raw(fields.max_dt_raw.unwrap_or_else(|| MAX_DT.to_raw()));
                let min_g =
                    Fixed::from_raw(fields.min_g_raw.unwrap_or(Fixed::from_int(1).to_raw()));
                let max_g =
                    Fixed::from_raw(fields.max_g_raw.unwrap_or(Fixed::from_int(1000).to_raw()));

                // Key schedule expansion (optional, use defaults)
                let expansion_factor = fields.expansion_factor.unwrap_or(1);
                let max_bytes_per_key = fields.max_bytes_per_key.unwrap_or(1 << 32);

                OrbitalConfig::new_full_ext(
                    bodies,
                    total_steps,
                    reseed_interval,
                    dt,
                    softening,
                    g,
                    min_separation,
                    ejection_energy_threshold,
                    monitor_interval,
                    min_bodies,
                    max_bodies,
                    min_dt,
                    max_dt,
                    min_g,
                    max_g,
                    expansion_factor,
                    max_bytes_per_key,
                )
                .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_struct(
            "OrbitalConfig",
            &[
                "bodies",
                "total_steps",
                "reseed_interval",
                "dt",
                "softening",
                "g",
                "min_separation",
                "ejection_energy_threshold",
                "monitor_interval",
                "min_bodies",
                "max_bodies",
                "min_dt",
                "max_dt",
                "min_g",
                "max_g",
            ],
            ConfigVisitor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::Fixed;

    fn valid_config() -> OrbitalConfig {
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
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        )
        .unwrap()
    }

    #[test]
    fn test_valid_config() {
        let config = valid_config();
        assert_eq!(config.bodies.len(), 5);
    }

    #[test]
    fn test_too_few_bodies() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::ZERO,
        );
        let result = OrbitalConfig::new(
            vec![sun, planet],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::TooFewBodies { count, min } => {
                assert_eq!(count, 2);
                assert_eq!(min, 5);
            },
            other => panic!("expected TooFewBodies, got: {:?}", other),
        }
    }

    #[test]
    fn test_identical_positions_rejected() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let sun2 = OrbitalBody::new(Fixed::from_raw(1 << 54), Vec3::ZERO, Vec3::ZERO);
        let p1 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        let p2 = OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        );
        let p3 = OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        );
        let result = OrbitalConfig::new(
            vec![sun, sun2, p1, p2, p3],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::IdenticalPositions { body_i, body_j } => {
                assert_eq!(body_i, 0);
                assert_eq!(body_j, 1);
            },
            other => panic!("expected IdenticalPositions, got: {:?}", other),
        }
    }

    #[test]
    fn test_initial_collapse_rejected() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        // All bodies at rest (zero velocity) to avoid ejection
        let p1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::from_int(10), Fixed::ZERO, Fixed::ZERO),
            Vec3::ZERO,
        );
        let p2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(10), Fixed::ZERO),
            Vec3::ZERO,
        );
        let p3 = OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(-10), Fixed::from_int(-10), Fixed::ZERO),
            Vec3::ZERO,
        );
        // Place planet4 extremely close to planet3 (different x, same y/z)
        // Use a clearly distinct position that is very close but not identical.
        // The offset is 1/1000 AU = Fixed::from_raw(SCALE / 1000) = Fixed::from_raw(1 << 64 / 1000)
        // which is approximately Fixed::from_raw(18446744073709551)
        let p4 = OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(
                Fixed::from_int(-10) + Fixed::from_raw(18446744073709551i128),
                Fixed::from_int(-10),
                Fixed::ZERO,
            ),
            Vec3::ZERO,
        );
        let result = OrbitalConfig::new_full(
            vec![sun, p1, p2, p3, p4],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
            Fixed::from_int(1), // 1 AU min_separation to trigger collapse (distance is ~0.001 AU)
            Fixed::from_raw(1 << 20),
            1000,
            5,
            100,
            kelvin_core::MIN_DT,
            kelvin_core::MAX_DT,
            Fixed::from_int(1),
            Fixed::from_int(1000),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InitialCollapse { body_i, body_j, .. } => {
                assert!(body_i < body_j);
            },
            other => panic!("expected InitialCollapse, got: {:?}", other),
        }
    }

    #[test]
    fn test_initial_ejection_rejected() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let p1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::from_int(100), Fixed::ZERO, Fixed::ZERO),
        ); // Escape velocity
        let p2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        );
        let p3 = OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        );
        let p4 = OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
        );
        let result = OrbitalConfig::new(
            vec![sun, p1, p2, p3, p4],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InitialEjection { body_index } => {
                assert_eq!(body_index, 1);
            },
            other => panic!("expected InitialEjection, got: {:?}", other),
        }
    }

    #[test]
    fn test_non_positive_mass() {
        let sun = OrbitalBody::new(Fixed::ZERO, Vec3::ZERO, Vec3::ZERO); // Zero mass
        let p1 = OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        );
        let p2 = OrbitalBody::new(
            Fixed::from_raw(1 << 53),
            Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO),
            Vec3::new(Fixed::from_int(-4), Fixed::ZERO, Fixed::ZERO),
        );
        let p3 = OrbitalBody::new(
            Fixed::from_raw(1 << 52),
            Vec3::new(Fixed::from_int(-1), Fixed::from_int(-1), Fixed::ZERO),
            Vec3::new(Fixed::from_int(3), Fixed::from_int(-2), Fixed::ZERO),
        );
        let p4 = OrbitalBody::new(
            Fixed::from_raw(1 << 51),
            Vec3::new(Fixed::from_int(2), Fixed::from_int(-1), Fixed::from_int(1)),
            Vec3::new(Fixed::from_int(-2), Fixed::from_int(3), Fixed::ZERO),
        );
        let result = OrbitalConfig::new(
            vec![sun, p1, p2, p3, p4],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::NonPositiveMass { body_index } => {
                assert_eq!(body_index, 0);
            },
            other => panic!("expected NonPositiveMass, got: {:?}", other),
        }
    }

    #[test]
    fn test_zero_steps() {
        let config = valid_config();
        let result = OrbitalConfig::new_full(
            config.bodies.clone(),
            0, // Zero steps
            config.reseed_interval,
            config.dt,
            config.softening,
            config.g,
            config.min_separation,
            config.ejection_energy_threshold,
            config.monitor_interval,
            config.min_bodies,
            config.max_bodies,
            config.min_dt,
            config.max_dt,
            config.min_g,
            config.max_g,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::ZeroSteps => {},
            other => panic!("expected ZeroSteps, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_dt() {
        let config = valid_config();
        let result = OrbitalConfig::new_full(
            config.bodies.clone(),
            config.total_steps,
            config.reseed_interval,
            Fixed::from_raw(1 << 10), // Way too small
            config.softening,
            config.g,
            config.min_separation,
            config.ejection_energy_threshold,
            config.monitor_interval,
            config.min_bodies,
            config.max_bodies,
            config.min_dt,
            config.max_dt,
            config.min_g,
            config.max_g,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidDt { .. } => {},
            other => panic!("expected InvalidDt, got: {:?}", other),
        }
    }

    #[test]
    fn test_invalid_g() {
        let config = valid_config();
        let result = OrbitalConfig::new_full(
            config.bodies.clone(),
            config.total_steps,
            config.reseed_interval,
            config.dt,
            config.softening,
            Fixed::from_int(0), // G = 0, below min_g = 1
            config.min_separation,
            config.ejection_energy_threshold,
            config.monitor_interval,
            config.min_bodies,
            config.max_bodies,
            config.min_dt,
            config.max_dt,
            config.min_g,
            config.max_g,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidG { .. } => {},
            other => panic!("expected InvalidG, got: {:?}", other),
        }
    }

    #[test]
    fn test_binary_roundtrip() {
        let config = valid_config();
        let binary = config.to_binary();
        let restored = OrbitalConfig::from_binary(&binary).unwrap();
        assert_eq!(config.bodies.len(), restored.bodies.len());
        assert_eq!(config.total_steps, restored.total_steps);
        assert_eq!(config.reseed_interval, restored.reseed_interval);
        assert_eq!(config.dt.to_raw(), restored.dt.to_raw());
        assert_eq!(config.softening.to_raw(), restored.softening.to_raw());
        assert_eq!(config.g.to_raw(), restored.g.to_raw());
        assert_eq!(config.min_separation.to_raw(), restored.min_separation.to_raw());
        assert_eq!(
            config.ejection_energy_threshold.to_raw(),
            restored.ejection_energy_threshold.to_raw()
        );
        assert_eq!(config.monitor_interval, restored.monitor_interval);
        assert_eq!(config.min_bodies, restored.min_bodies);
        assert_eq!(config.max_bodies, restored.max_bodies);
        assert_eq!(config.min_dt.to_raw(), restored.min_dt.to_raw());
        assert_eq!(config.max_dt.to_raw(), restored.max_dt.to_raw());
        assert_eq!(config.min_g.to_raw(), restored.min_g.to_raw());
        assert_eq!(config.max_g.to_raw(), restored.max_g.to_raw());
        assert_eq!(config.expansion_factor, restored.expansion_factor);
        assert_eq!(config.max_bytes_per_key, restored.max_bytes_per_key);
    }

    #[test]
    fn test_expansion_factor_default() {
        let config = valid_config();
        assert_eq!(config.expansion_factor, 1);
    }

    #[test]
    fn test_max_bytes_per_key_default() {
        let config = valid_config();
        assert_eq!(config.max_bytes_per_key, 1 << 32);
    }

    #[test]
    fn test_expansion_factor_validation() {
        let config = valid_config();
        let result = OrbitalConfig::new_full_ext(
            config.bodies.clone(),
            config.total_steps,
            config.reseed_interval,
            config.dt,
            config.softening,
            config.g,
            config.min_separation,
            config.ejection_energy_threshold,
            config.monitor_interval,
            config.min_bodies,
            config.max_bodies,
            config.min_dt,
            config.max_dt,
            config.min_g,
            config.max_g,
            0, // invalid: < 1
            config.max_bytes_per_key,
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidExpansionFactor(0) => {},
            other => panic!("expected InvalidExpansionFactor(0), got: {:?}", other),
        }
    }

    #[test]
    fn test_max_bytes_per_key_validation() {
        let config = valid_config();
        let result = OrbitalConfig::new_full_ext(
            config.bodies.clone(),
            config.total_steps,
            config.reseed_interval,
            config.dt,
            config.softening,
            config.g,
            config.min_separation,
            config.ejection_energy_threshold,
            config.monitor_interval,
            config.min_bodies,
            config.max_bodies,
            config.min_dt,
            config.max_dt,
            config.min_g,
            config.max_g,
            1,
            0, // invalid: < 1
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidMaxBytesPerKey(0) => {},
            other => panic!("expected InvalidMaxBytesPerKey(0), got: {:?}", other),
        }
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_json_roundtrip() {
        let config = valid_config();
        let json = config.to_json().unwrap();
        let restored = OrbitalConfig::from_json(&json).unwrap();
        assert_eq!(config.bodies.len(), restored.bodies.len());
        assert_eq!(config.total_steps, restored.total_steps);
        assert_eq!(config.min_separation.to_raw(), restored.min_separation.to_raw());
        assert_eq!(config.min_bodies, restored.min_bodies);
        assert_eq!(config.max_bodies, restored.max_bodies);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_json_without_new_fields_still_works() {
        // Old-format JSON without the new fields should still parse with defaults
        let old_json = r#"{
            "bodies": [
                {"mass": 1099511627776, "position": {"x": 0, "y": 0, "z": 0}, "velocity": {"x": 0, "y": 0, "z": 0}},
                {"mass": 1099511627776, "position": {"x": 18446744073709551616, "y": 0, "z": 0}, "velocity": {"x": 0, "y": 0, "z": 0}},
                {"mass": 1099511627776, "position": {"x": 0, "y": 36893488147419103232, "z": 0}, "velocity": {"x": 0, "y": 0, "z": 0}},
                {"mass": 1099511627776, "position": {"x": -18446744073709551616, "y": -18446744073709551616, "z": 0}, "velocity": {"x": 0, "y": 0, "z": 0}},
                {"mass": 1099511627776, "position": {"x": 36893488147419103232, "y": -18446744073709551616, "z": 18446744073709551616}, "velocity": {"x": 0, "y": 0, "z": 0}}
            ],
            "total_steps": 10000,
            "reseed_interval": 1000,
            "dt": 18014398509481984,
            "softening": 17592186044416,
            "g": 18446744073709551616,
            "min_g": 18446744073709551616,
            "max_g": 18446744073709551616000000
        }"#;
        let config = OrbitalConfig::from_json(old_json).unwrap();
        assert_eq!(config.bodies.len(), 5);
        assert_eq!(config.min_separation.to_raw(), MIN_SEPARATION.to_raw());
        assert_eq!(config.min_bodies, MIN_BODIES);
        assert_eq!(config.max_bodies, MAX_BODIES);
    }

    #[test]
    fn test_min_bodies_validation() {
        let result = OrbitalConfig::new_full(
            vec![],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
            MIN_SEPARATION,
            EJECTION_ENERGY_THRESHOLD,
            MONITOR_INTERVAL,
            0, // min_bodies = 0 (invalid)
            100,
            kelvin_core::MIN_DT,
            kelvin_core::MAX_DT,
            Fixed::from_int(1),
            Fixed::from_int(1000),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidMinBodies(0) => {},
            other => panic!("expected InvalidMinBodies(0), got: {:?}", other),
        }
    }

    #[test]
    fn test_max_bodies_less_than_min() {
        let result = OrbitalConfig::new_full(
            vec![],
            10000,
            1000,
            kelvin_core::DEFAULT_DT,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
            MIN_SEPARATION,
            EJECTION_ENERGY_THRESHOLD,
            MONITOR_INTERVAL,
            10, // min = 10
            5,  // max = 5 (invalid)
            kelvin_core::MIN_DT,
            kelvin_core::MAX_DT,
            Fixed::from_int(1),
            Fixed::from_int(1000),
        );
        assert!(result.is_err());
        match result.unwrap_err() {
            ConfigError::InvalidMaxBodies { max, min } => {
                assert_eq!(max, 5);
                assert_eq!(min, 10);
            },
            other => panic!("expected InvalidMaxBodies, got: {:?}", other),
        }
    }
}
