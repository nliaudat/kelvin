//! # kelvin-ffi
//!
//! C FFI bindings for the Kelvin cryptosystem.
//!
//! Provides a C ABI for integration with iOS, Android, and embedded systems.
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**

// FFI inherently requires unsafe code
#![allow(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod c_api;

pub use c_api::*;
