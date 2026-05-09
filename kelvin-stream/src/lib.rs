//! # kelvin-stream
//!
//! Stream cipher integration for the Kelvin cryptosystem.
//!
//! Provides:
//! - `StreamCipher` trait
//! - `ChaChaStream` — ChaCha20 wrapper with rekeying support
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**

#![deny(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod chacha;
mod traits;

pub use chacha::ChaChaStream;
pub use traits::StreamCipher;
