//! Cipher mode trait and zero-sized marker types.
//!
//! Inspired by the dalek cryptography suite's approach of encoding protocol
//! properties in the type system (e.g., `SigningKey` vs `VerifyingKey` in
//! ed25519-dalek, or parameterized types in `bulletproofs`).
//!
//! # Type-level Mode Safety
//!
//! Each mode is a zero-sized marker type implementing [`Mode`]. This lets you
//! write generic functions that work across modes while getting compile-time
//! guarantees about which operations are valid:
//!
//! ```rust,ignore
//! use kelvin::{Mode, Secure, Chaos};
//!
//! fn encrypt_in_mode<M: Mode>(config: OrbitalConfig, data: &mut [u8])
//!     -> Result<(), KelvinError>
//! {
//!     let mut cipher = Kelvin::<M>::new(config)?;
//!     cipher.encrypt(data)?;
//!     Ok(())
//! }
//! ```
//!
//! # Mode Properties
//!
//! | Mode | Marker | Authenticated | Unlimited Keystream | Cipher |
//! |------|--------|:------------:|:-------------------:|--------|
//! | V1 Secure | [`Secure`] | ✅ AEAD | ❌ Finite | ChaCha20Poly1305 |
//! | V2 Chaos | [`Chaos`] | ❌ | ✅ Unlimited | SHAKE256 XOR |
//! | V3 Photon | [`Photon`] | ❌ | ❌ Finite | SHAKE256 XOR |
//! | H Quantum | [`Quantum`] | ❌ | ✅ ≈Unlimited | SHAKE256 XOR |
//! | — Prism | [`Prism`] | ❌ | ❌ Finite | SHAKE256 XOR |
//! | — Split | [`Split`] | ❌ | ❌ Finite | SHAKE256 XOR |
//! | — Flare | [`Flare`] | ❌ | ❌ Finite | SHAKE256 XOR |
//!
//! # What This Crate Is Not For
//!
//! This module does **not** provide runtime dispatch or dynamic polymorphism
//! across modes. It is purely a compile-time type-level mechanism. If you
//! need to choose a mode at runtime, use the concrete struct types directly
//! (e.g., [`KelvinStreaming`](crate::KelvinStreaming), [`KelvinPhoton`](crate::KelvinPhoton)).

use crate::error::KelvinError;
use kelvin_core::IntegrationMethod;
use kelvin_kdf::OrbitalConfig;
use kelvin_stream::StreamCipher;

/// Cipher mode trait for the Kelvin cryptosystem.
///
/// Each mode is a distinct zero-sized marker type. The associated types and
/// constants define the mode's cryptographic properties at compile time.
///
/// # Associated Types
///
/// * `Cipher` — The stream cipher type used by this mode.
///
/// # Associated Constants
///
/// * `AUTHENTICATED` — Whether this mode provides AEAD authentication.
/// * `UNLIMITED_KEYSTREAM` — Whether this mode can produce unlimited keystream.
/// * `DESCRIPTION` — A human-readable name for this mode.
///
/// # When You Should Use This Trait
///
/// * Writing generic functions that work with multiple modes
/// * Building type-level mode-selection logic
/// * Enforcing mode-specific constraints at compile time
///
/// # When You Should NOT Use This Trait
///
/// * If you need runtime mode selection — use the concrete struct types directly
/// * If you only ever use one mode — just use the concrete struct
pub trait Mode: Sized + Send + Sync {
    /// The stream cipher type used by this mode.
    type Cipher: StreamCipher + Send + Sync;

    /// Whether this mode provides AEAD authentication.
    const AUTHENTICATED: bool;

    /// Whether this mode can produce unlimited keystream.
    const UNLIMITED_KEYSTREAM: bool;

    /// Human-readable description of this mode.
    const DESCRIPTION: &'static str;

    /// Create the cipher state for this mode from an orbital config.
    ///
    /// This is the mode-level initialization method. It validates the config,
    /// runs the orbital simulation (or delegates to the mode's own init),
    /// and produces a boxed stream cipher ready for encryption/decryption.
    fn init(config: OrbitalConfig, method: IntegrationMethod) -> Result<Self::Cipher, KelvinError>;
}

// ── Marker types ──────────────────────────────────────────────────────────

/// **V1 Secure** — ChaCha20Poly1305 AEAD with virtual key schedule.
///
/// Properties:
/// * ✅ Authenticated (AEAD)
/// * ❌ Finite keystream (~28 GiB)
/// * 🐢 ~500 MB/s
///
/// Use for: General purpose encryption when authentication is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Secure;

/// **V2 Chaos** — Per-step OTP streaming with one simulation step per chunk.
///
/// Properties:
/// * ❌ Unauthenticated (XOR is malleable)
/// * ✅ Unlimited keystream
/// * 🐌 ~3 MB/s
///
/// Use for: Streaming, real-time encryption where unlimited keystream is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Chaos;

/// **V3 Photon** — Fast bulk OTP via HKDF→SHAKE256 XOR.
///
/// Properties:
/// * ❌ Unauthenticated (XOR is malleable)
/// * ❌ Finite keystream
/// * 🚀 ~5 GB/s
///
/// Use for: Bulk encryption where authentication is handled externally.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Photon;

/// **H Quantum** — Hybrid OTP (V3+V2 XOR with orbital reseeding).
///
/// Properties:
/// * ❌ Unauthenticated (XOR is malleable)
/// * ✅ ≈Unlimited keystream (orbital reseeding)
/// * 🚀 ~5 GB/s
///
/// Use for: Best all-around mode when authentication is not needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Quantum;

/// **Prism** — Standalone OTP key generator for homomorphic encryption.
///
/// Properties:
/// * ❌ Unauthenticated
/// * ❌ Finite keystream
/// * 🚀 ~5 GB/s
///
/// Use for: Generating OTP keys for HE recryption and split-key operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Prism;

/// **Split** — Dedicated XOR key-splitter for homomorphic encryption.
///
/// Properties:
/// * ❌ Unauthenticated
/// * ❌ Finite keystream
/// * 🚀 ~5 GB/s
///
/// Use for: XOR-based key splitting for HE workflows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Split;

/// **Flare** — Chaotic FHE secret key generator.
///
/// Properties:
/// * ❌ Unauthenticated
/// * ❌ Finite keystream
/// * 🚀 ~5 GB/s
///
/// Use for: Generating high-entropy FHE secret keys (BFV, CKKS, TFHE).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Flare;

// Note: Full `Mode` trait implementations (`impl Mode for Secure`, etc.) would
// require refactoring the existing structs (`Kelvin`, `KelvinStreaming`, etc.)
// to be generic over `M: Mode`. This is a significant refactor that involves
// changing the core API. The marker types and trait definition are provided as
// the foundation — users can adopt them incrementally.
//
// For now, the marker types serve as documentation-level mode identifiers
// and compile-time mode metadata. Users continue to use the concrete struct
// types (e.g., `KelvinStreaming`, `KelvinPhoton`) directly.
