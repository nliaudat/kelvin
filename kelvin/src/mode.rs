//! Encryption mode marker types.
//!
//! Defines the `Mode` trait and zero-sized marker types for all Kelvin
//! encryption modes. Inspired by dalek cryptography's type-level protocol
//! encoding.

/// Trait for Kelvin encryption modes.
///
/// Implemented by zero-sized marker types that encode the mode's
/// cryptographic properties at the type level.
pub trait Mode: sealed::Sealed + std::fmt::Debug {
    /// Human-readable cipher description.
    const CIPHER: &'static str;
    /// Whether this mode provides authentication.
    const AUTHENTICATED: bool;
    /// Whether this mode has unlimited keystream.
    const UNLIMITED_KEYSTREAM: bool;
    /// Human-readable description of this mode.
    const DESCRIPTION: &'static str;
}

mod sealed {
    pub trait Sealed {}
    impl Sealed for super::Secure {}
    impl Sealed for super::Chaos {}
    impl Sealed for super::Photon {}
    impl Sealed for super::Quantum {}
    impl Sealed for super::Prism {}
    impl Sealed for super::Split {}
    impl Sealed for super::Flare {}
}

/// **V1 Secure** — Authenticated AEAD via ChaCha20Poly1305.
///
/// Finite keystream (~28 GiB per key). Provides built-in authentication
/// (AEAD tag). Best for general-purpose encryption where integrity
/// is required.
#[derive(Debug)]
pub struct Secure;
impl Mode for Secure {
    const CIPHER: &'static str = "ChaCha20Poly1305 (AEAD)";
    const AUTHENTICATED: bool = true;
    const UNLIMITED_KEYSTREAM: bool = false;
    const DESCRIPTION: &'static str =
        "V1 Secure — General purpose with built-in authentication. Finite keystream.";
}

/// **V2 Chaos** — Per-step stream cipher with one simulation step per chunk.
///
/// Unlimited keystream (keep simulating). Provides XOR encryption
/// via SHAKE256. No built-in authentication.
#[derive(Debug)]
pub struct Chaos;
impl Mode for Chaos {
    const CIPHER: &'static str = "SHAKE256 XOR (per-step streaming)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = true;
    const DESCRIPTION: &'static str =
        "V2 Chaos — Per-step streaming with one simulation step per chunk. Unlimited keystream.";
}

/// **V3 Photon** — Fast bulk stream cipher via HKDF→SHAKE256 XOR.
///
/// Finite keystream (bounded by key schedule). Provides XOR encryption
/// via HKDF→SHAKE256. No built-in authentication.
#[derive(Debug)]
pub struct Photon;
impl Mode for Photon {
    const CIPHER: &'static str = "HKDF→SHAKE256 XOR (batch streaming)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = false;
    const DESCRIPTION: &'static str =
        "V3 Photon — Fast bulk stream cipher via HKDF→SHAKE256 XOR. Finite keystream.";
}

/// **H Quantum** — Hybrid stream cipher (V3+V2 XOR with orbital reseeding).
///
/// Effectively unlimited keystream (periodic orbital reseeding).
/// Provides XOR encryption via SHAKE256 with fresh chaotic entropy.
/// No built-in authentication.
#[derive(Debug)]
pub struct Quantum;
impl Mode for Quantum {
    const CIPHER: &'static str = "SHAKE256 XOR (hybrid cache + orbital reseed)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = true;
    const DESCRIPTION: &'static str =
        "H Quantum — Hybrid stream cipher (V3+V2 XOR with orbital reseeding). Effectively unlimited.";
}

/// **Prism** — Standalone stream key generator for homomorphic encryption.
///
/// Provides domain-separated stream key material for HE recryption,
/// split-key XOR homomorphism, and chaotic FHE key generation.
/// No built-in authentication.
#[derive(Debug)]
pub struct Prism;
impl Mode for Prism {
    const CIPHER: &'static str = "HKDF→SHAKE256 XOR (HE stream key generator)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = true;
    const DESCRIPTION: &'static str =
        "Prism — Standalone stream key generator for homomorphic encryption.";
}

/// **Split** — Dedicated XOR key-splitter for homomorphic encryption.
///
/// Provides domain-separated split-key XOR homomorphism.
/// No built-in authentication.
#[derive(Debug)]
pub struct Split;
impl Mode for Split {
    const CIPHER: &'static str = "HKDF→SHAKE256 XOR (split-key generator)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = true;
    const DESCRIPTION: &'static str =
        "Split — Dedicated XOR key-splitter for homomorphic encryption.";
}

/// **Flare** — Chaotic FHE secret key generator.
///
/// Provides domain-separated FHE secret key generation.
/// No built-in authentication.
#[derive(Debug)]
pub struct Flare;
impl Mode for Flare {
    const CIPHER: &'static str = "HKDF→SHAKE256 XOR (FHE key generator)";
    const AUTHENTICATED: bool = false;
    const UNLIMITED_KEYSTREAM: bool = true;
    const DESCRIPTION: &'static str =
        "Flare — Chaotic FHE secret key generator (30 DOF n-body chaos).";
}