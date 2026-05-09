//! Error types for the Kelvin cryptosystem.

use thiserror::Error;

/// Errors that can occur during Kelvin encryption/decryption.
#[derive(Error, Debug)]
pub enum KelvinError {
    /// Configuration failed validation.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Lyapunov time insufficient for requested steps.
    #[error("insufficient Lyapunov time: requested {requested} steps, safe {safe}")]
    InsufficientLyapunovTime {
        /// Number of simulation steps requested.
        requested: u64,
        /// Maximum safe steps based on Lyapunov estimation.
        safe: u64,
    },

    /// Orbital simulation time exhausted.
    #[error("orbital simulation time exhausted at step {step}")]
    OrbitalTimeExhausted {
        /// The step at which exhaustion occurred.
        step: u64,
    },

    /// Seed material exhausted.
    #[error("seed material exhausted")]
    SeedExhausted,

    /// I/O error during encryption/decryption.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<kelvin_kdf::ConfigError> for KelvinError {
    fn from(e: kelvin_kdf::ConfigError) -> Self {
        KelvinError::InvalidConfig(e.to_string())
    }
}

impl From<kelvin_kdf::LyapunovError> for KelvinError {
    fn from(e: kelvin_kdf::LyapunovError) -> Self {
        KelvinError::InvalidConfig(e.to_string())
    }
}
