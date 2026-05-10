//! Orbital configuration — the shared secret format.
//!
//! The `OrbitalConfig` defines the initial conditions for the n-body
//! simulation. It is the shared secret between communicating parties.

use alloc::vec::Vec;
use core::fmt;

use kelvin_core::{Fixed, OrbitalBody, Vec3};
#[allow(unused_imports)]
use kelvin_core::{MIN_BODIES, MAX_BODIES, DEFAULT_RESEED_INTERVAL};

/// Orbital configuration — the shared secret.
///
/// Contains:
/// - Initial positions, velocities, and masses of all bodies
/// - Simulation parameters (steps, dt, softening)
/// - Reseed interval for the key schedule
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
}

impl OrbitalConfig {
    /// Create a new orbital configuration.
    ///
    /// Returns an error if validation fails.
    pub fn new(
        bodies: Vec<OrbitalBody>,
        total_steps: u64,
        reseed_interval: u64,
        dt: Fixed,
        softening: Fixed,
        g: Fixed,
    ) -> Result<Self, ConfigError> {
        let config = OrbitalConfig {
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
        };
        config.validate()?;
        Ok(config)
    }

    /// Validate the configuration.
    ///
    /// Checks:
    /// - Number of bodies is within [MIN_BODIES, MAX_BODIES]
    /// - All masses are positive
    /// - dt and softening are positive
    /// - total_steps > 0
    /// - reseed_interval > 0 and <= total_steps
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.bodies.len() < MIN_BODIES {
            return Err(ConfigError::TooFewBodies {
                count: self.bodies.len(),
                min: MIN_BODIES,
            });
        }
        if self.bodies.len() > MAX_BODIES {
            return Err(ConfigError::TooManyBodies {
                count: self.bodies.len(),
                max: MAX_BODIES,
            });
        }
        for (i, body) in self.bodies.iter().enumerate() {
            if body.mass <= Fixed::ZERO {
                return Err(ConfigError::NonPositiveMass { body_index: i });
            }
        }
        if self.dt <= Fixed::ZERO {
            return Err(ConfigError::InvalidDt);
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
        if self.g < Fixed::from_int(1) || self.g > Fixed::from_int(1000) {
            return Err(ConfigError::InvalidG);
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
    pub fn to_binary(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(4 + self.bodies.len() * 112 + 8 + 8 + 16 + 16 + 16);

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

        buf
    }

    /// Deserialize from binary format.
    pub fn from_binary(data: &[u8]) -> Result<Self, ConfigError> {
        let mut offset = 0;

        if data.len() < 4 {
            return Err(ConfigError::InvalidBinary("data too short".into()));
        }

        let n_bodies = u32::from_le_bytes([
            data[offset], data[offset + 1], data[offset + 2], data[offset + 3],
        ]) as usize;
        offset += 4;

        let body_size = 112; // 7 × i128
        let header_size = 4 + n_bodies * body_size + 8 + 8 + 16 + 16 + 16;

        if data.len() < header_size {
            return Err(ConfigError::InvalidBinary("data too short for bodies".into()));
        }

        let mut bodies = Vec::with_capacity(n_bodies);
        for _ in 0..n_bodies {
            let mass = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;

            let px = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;
            let py = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;
            let pz = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;

            let vx = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;
            let vy = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;
            let vz = Fixed::from_raw(i128::from_le_bytes(
                data[offset..offset + 16].try_into().unwrap(),
            ));
            offset += 16;

            bodies.push(OrbitalBody::new(
                mass,
                Vec3::new(px, py, pz),
                Vec3::new(vx, vy, vz),
            ));
        }

        let total_steps = u64::from_le_bytes(
            data[offset..offset + 8].try_into().unwrap(),
        );
        offset += 8;

        let reseed_interval = u64::from_le_bytes(
            data[offset..offset + 8].try_into().unwrap(),
        );
        offset += 8;

        let dt = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16].try_into().unwrap(),
        ));
        offset += 16;

        let softening = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16].try_into().unwrap(),
        ));
        offset += 16;

        let g = Fixed::from_raw(i128::from_le_bytes(
            data[offset..offset + 16].try_into().unwrap(),
        ));

        let config = OrbitalConfig {
            bodies,
            total_steps,
            reseed_interval,
            dt,
            softening,
            g,
        };
        config.validate()?;
        Ok(config)
    }
}

/// Errors from configuration validation.
#[derive(Clone, Debug)]
pub enum ConfigError {
    /// Too few bodies.
    TooFewBodies {
        /// Number of bodies provided.
        count: usize,
        /// Minimum required.
        min: usize,
    },
    /// Too many bodies.
    TooManyBodies {
        /// Number of bodies provided.
        count: usize,
        /// Maximum allowed.
        max: usize,
    },
    /// A body has non-positive mass.
    NonPositiveMass {
        /// Index of the offending body.
        body_index: usize,
    },
    /// Time step must be positive.
    InvalidDt,
    /// Softening factor must be positive.
    InvalidSoftening,
    /// Total steps must be > 0.
    ZeroSteps,
    /// Reseed interval must be > 0 and <= total_steps.
    InvalidReseedInterval,
    /// Gravitational constant must be within [1, 1000].
    InvalidG,
    /// Serialization error.
    Serialization(String),
    /// Invalid binary data.
    InvalidBinary(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::TooFewBodies { count, min } => {
                write!(f, "too few bodies: {count}, need at least {min}")
            }
            ConfigError::TooManyBodies { count, max } => {
                write!(f, "too many bodies: {count}, max is {max}")
            }
            ConfigError::NonPositiveMass { body_index } => {
                write!(f, "body {body_index} has non-positive mass")
            }
            ConfigError::InvalidDt => write!(f, "time step must be positive"),
            ConfigError::InvalidSoftening => write!(f, "softening factor must be positive"),
            ConfigError::ZeroSteps => write!(f, "total steps must be > 0"),
            ConfigError::InvalidReseedInterval => {
                write!(f, "reseed interval must be > 0 and <= total_steps")
            }
            ConfigError::InvalidG => write!(f, "gravitational constant must be within [1, 1000]"),
            ConfigError::Serialization(msg) => write!(f, "serialization error: {msg}"),
            ConfigError::InvalidBinary(msg) => write!(f, "invalid binary data: {msg}"),
        }
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for OrbitalConfig {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("OrbitalConfig", 6)?;
        state.serialize_field("bodies", &self.bodies)?;
        state.serialize_field("total_steps", &self.total_steps)?;
        state.serialize_field("reseed_interval", &self.reseed_interval)?;
        state.serialize_field("dt", &self.dt.to_raw())?;
        state.serialize_field("softening", &self.softening.to_raw())?;
        state.serialize_field("g", &self.g.to_raw())?;
        state.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for OrbitalConfig {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::{self, MapAccess, Visitor};
        use core::fmt;

        #[derive(Default)]
        struct ConfigFields {
            bodies: Option<Vec<OrbitalBody>>,
            total_steps: Option<u64>,
            reseed_interval: Option<u64>,
            dt_raw: Option<i128>,
            softening_raw: Option<i128>,
            g_raw: Option<i128>,
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
                        _ => { let _: serde_json::Value = map.next_value()?; }
                    }
                }

                let bodies = fields.bodies.ok_or_else(|| de::Error::missing_field("bodies"))?;
                let total_steps = fields.total_steps.ok_or_else(|| de::Error::missing_field("total_steps"))?;
                let reseed_interval = fields.reseed_interval.unwrap_or(DEFAULT_RESEED_INTERVAL);
                let dt = Fixed::from_raw(fields.dt_raw.unwrap_or_else(|| kelvin_core::DEFAULT_DT.to_raw()));
                let softening = Fixed::from_raw(fields.softening_raw.unwrap_or_else(|| kelvin_core::SOFTENING_FACTOR.to_raw()));
                let g = Fixed::from_raw(fields.g_raw.unwrap_or_else(|| kelvin_core::DEFAULT_G.to_raw()));

                OrbitalConfig::new(bodies, total_steps, reseed_interval, dt, softening, g)
                    .map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_struct("OrbitalConfig", &["bodies", "total_steps", "reseed_interval", "dt", "softening", "g"], ConfigVisitor)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use kelvin_core::Fixed;

    fn valid_config() -> OrbitalConfig {
        let sun = OrbitalBody::new(
            Fixed::ONE,
            Vec3::ZERO,
            Vec3::ZERO,
        );
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
            10000,
            1000,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        ).unwrap()
    }

    #[test]
    fn test_valid_config() {
        let config = valid_config();
        assert_eq!(config.bodies.len(), 3);
    }

    #[test]
    fn test_too_few_bodies() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(Fixed::from_raw(1 << 54), Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
        let result = OrbitalConfig::new(
            vec![sun, planet],
            10000, 1000,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(matches!(result, Err(ConfigError::TooFewBodies { .. })));
    }

    #[test]
    fn test_non_positive_mass() {
        let body = OrbitalBody::new(
            Fixed::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
        );
        let result = OrbitalConfig::new(
            vec![body, body, body],
            10000, 1000,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(matches!(result, Err(ConfigError::NonPositiveMass { .. })));
    }

    #[test]
    fn test_zero_steps() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(Fixed::from_raw(1 << 54), Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
        let planet2 = OrbitalBody::new(Fixed::from_raw(1 << 53), Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO), Vec3::ZERO);
        let result = OrbitalConfig::new(
            vec![sun, planet, planet2],
            0, 1000,
            Fixed::from_raw(1 << 44),
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(matches!(result, Err(ConfigError::ZeroSteps)));
    }

    #[test]
    fn test_binary_roundtrip() {
        let config = valid_config();
        let binary = config.to_binary();
        let decoded = OrbitalConfig::from_binary(&binary).unwrap();
        assert_eq!(config.bodies.len(), decoded.bodies.len());
        assert_eq!(config.total_steps, decoded.total_steps);
        assert_eq!(config.reseed_interval, decoded.reseed_interval);
        assert_eq!(config.dt, decoded.dt);
        assert_eq!(config.softening, decoded.softening);
        assert_eq!(config.g, decoded.g);
        for (a, b) in config.bodies.iter().zip(decoded.bodies.iter()) {
            assert_eq!(a.mass, b.mass);
            assert_eq!(a.position, b.position);
            assert_eq!(a.velocity, b.velocity);
        }
    }

    #[test]
    fn test_binary_invalid_short() {
        let result = OrbitalConfig::from_binary(&[0, 0, 0]);
        assert!(matches!(result, Err(ConfigError::InvalidBinary(_))));
    }

    #[test]
    fn test_validate_dt() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(Fixed::from_raw(1 << 54), Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
        let planet2 = OrbitalBody::new(Fixed::from_raw(1 << 53), Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO), Vec3::ZERO);
        let result = OrbitalConfig::new(
            vec![sun, planet, planet2],
            10000, 1000,
            Fixed::ZERO,
            Fixed::from_raw(1 << 44),
            kelvin_core::DEFAULT_G,
        );
        assert!(matches!(result, Err(ConfigError::InvalidDt)));
    }

    #[test]
    fn test_validate_softening() {
        let sun = OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO);
        let planet = OrbitalBody::new(Fixed::from_raw(1 << 54), Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO), Vec3::ZERO);
        let planet2 = OrbitalBody::new(Fixed::from_raw(1 << 53), Vec3::new(Fixed::ZERO, Fixed::from_int(2), Fixed::ZERO), Vec3::ZERO);
        let result = OrbitalConfig::new(
            vec![sun, planet, planet2],
            10000, 1000,
            Fixed::from_raw(1 << 44),
            Fixed::ZERO,
            kelvin_core::DEFAULT_G,
        );
        assert!(matches!(result, Err(ConfigError::InvalidSoftening)));
    }
}
