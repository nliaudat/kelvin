//! # kelvin-kdf
//!
//! Key derivation layer for the Kelvin cryptosystem.
//!
//! Provides:
//! - `OrbitalConfig` — shared secret format with JSON/binary serialization
//! - `LyapunovEstimator` — shadow orbit method for Lyapunov time estimation
//! - `extract_seed` — SHA3-512 entropy extraction from orbital state
//! - `KeySchedule` — reseeding, step counting, and exhaustion detection
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**

#![deny(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

extern crate alloc;

mod config;
mod lyapunov;
mod extractor;
mod schedule;

pub use config::{OrbitalConfig, ConfigError};
pub use lyapunov::{LyapunovEstimator, LyapunovResult, LyapunovConfidence, LyapunovError};
pub use extractor::extract_seed;
pub use schedule::{KeySchedule, ScheduleState};
