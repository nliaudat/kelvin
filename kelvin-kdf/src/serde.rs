//! Serde serialization support for `OrbitalConfig`.
//!
//! Provides JSON serialization/deserialization via `serde_json` and
//! custom `Serialize`/`Deserialize` implementations that store `Fixed`
//! values as their raw `i128` representation.
//!
//! This module is only compiled when the `serde` feature is enabled.

use alloc::vec::Vec;

use kelvin_core::{
    Fixed, OrbitalBody, DEFAULT_DT, DEFAULT_G, DEFAULT_RESEED_INTERVAL, EJECTION_ENERGY_THRESHOLD,
    MAX_BODIES, MAX_DT, MIN_BODIES, MIN_DT, MIN_SEPARATION, MONITOR_INTERVAL, SOFTENING_FACTOR,
};

use crate::config::{ConfigError, OrbitalConfig};

impl OrbitalConfig {
    /// Serialize to JSON string.
    pub fn to_json(&self) -> Result<String, ConfigError> {
        serde_json::to_string(self).map_err(|e| ConfigError::Serialization(e.to_string()))
    }

    /// Deserialize from JSON string.
    pub fn from_json(json: &str) -> Result<Self, ConfigError> {
        serde_json::from_str(json).map_err(|e| ConfigError::Serialization(e.to_string()))
    }
}

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
