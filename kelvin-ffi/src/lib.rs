//! # kelvin-ffi
//!
//! C FFI bindings for the Kelvin cryptosystem.
//!
//! Provides a C ABI for integration with iOS, Android, and embedded systems.
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**
//!
//! ## References
//!
//! - Liaudat, N. (2025). "Kelvin: Orbital Chaos KDF Cryptosystem."
//!   GitHub: https://github.com/nliaudat/kelvin

// FFI inherently requires unsafe code for C interop
// Safety docs are provided in the C header comments; clippy's missing_safety_doc
// is too noisy for FFI functions that are documented at the module level.
// missing_docs is allowed because FFI functions are documented in the C header.
#![allow(unsafe_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(missing_docs)]
#![warn(missing_debug_implementations)]

pub mod c_api;

pub use c_api::*;
