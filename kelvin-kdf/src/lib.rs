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
//! ## What This Crate Is Not For
//!
//! This crate does **not** provide the orbital simulation engine itself
//! (see [`kelvin-core`]) or the top-level encryption API (see [`kelvin`]).
//! It is the intermediate key derivation layer. If you only need key
//! derivation from orbital seeds without running simulations, this is
//! the right crate. If you need fixed-point math or n-body integration,
//! use `kelvin-core` directly.
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**
//!
//! ## References
//!
//! - Benettin et al. (1980). "Lyapunov Characteristic Exponents for Smooth
//!   Dynamical Systems." *Meccanica*, 15, 9–20.
//! - Wolf et al. (1985). "Determining Lyapunov Exponents from a Time Series."
//!   *Physica D*, 16(3), 285–317.
//! - NIST FIPS PUB 202 (2015). "SHA-3 Standard."
//! - Bernstein (2008). "ChaCha, a Variant of Salsa20." *SASC 2008*.

#![deny(unsafe_code)]
#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

extern crate alloc;

mod asymmetric;
mod config;
mod extractor;
mod lyapunov;
mod orbital_state;
mod schedule;

#[cfg(feature = "serde")]
mod serde;

pub use config::{ConfigError, OrbitalConfig};
pub use extractor::{extract_seed, extract_shake256, extract_shake256_into};
pub use lyapunov::{LyapunovConfidence, LyapunovError, LyapunovEstimator, LyapunovResult};

pub use asymmetric::{AsymmetricError, OrbitalKeyPair};
pub use orbital_state::{OrbitalError, OrbitalState, MAX_VERLET_STEPS};
pub use schedule::{KeySchedule, ScheduleState};
