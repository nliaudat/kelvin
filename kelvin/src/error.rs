//! Error types for the Kelvin cryptosystem.

use thiserror::Error;

/// Errors that can occur during Kelvin encryption/decryption.
#[derive(Error, Debug)]
pub enum KelvinError {
    /// Configuration failed validation.
    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    /// Orbital simulation did not reach the chaotic regime.
    #[error(
        "insufficient chaos: requested {requested} steps, but Lyapunov horizon is at {horizon}"
    )]
    InsufficientChaos {
        /// Number of simulation steps requested.
        requested: u64,
        /// Step at which chaos (unpredictability) is reached.
        horizon: u64,
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

    /// AEAD authentication error (tampered ciphertext).
    #[error("AEAD authentication failed: {0}")]
    AeadError(String),

    /// Stability monitoring detected a system failure (ejection or collapse).
    #[error("stability error: {0}")]
    StabilityError(String),

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

impl From<aead::Error> for KelvinError {
    fn from(e: aead::Error) -> Self {
        KelvinError::AeadError(e.to_string())
    }
}

impl From<kelvin_core::StabilityError> for KelvinError {
    fn from(e: kelvin_core::StabilityError) -> Self {
        KelvinError::StabilityError(e.to_string())
    }
}
