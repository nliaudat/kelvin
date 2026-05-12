//! # kelvin-stream
//!
//! Stream cipher integration for the Kelvin cryptosystem.
//!
//! Provides:
//! - `StreamCipher` trait — authenticated encryption/decryption
//! - `ChaChaStream` — ChaCha20Poly1305 AEAD wrapper
//!
//! ## Security
//!
//! **EXPERIMENTAL — NOT FOR PRODUCTION USE.**
//!
//! ## References
//!
//! - Bernstein, D. J. (2008). "ChaCha, a Variant of Salsa20." *SASC 2008*.
//! - Nir, Y., & Langley, A. (2018). RFC 8439. doi:10.17487/RFC8439

#![deny(unsafe_code)]
#![forbid(unsafe_code)]
#![warn(missing_docs, missing_debug_implementations)]

mod chacha;
mod traits;

#[cfg(feature = "aes-ni")]
mod aes_ctr;

pub use chacha::ChaChaStream;
pub use traits::StreamCipher;

#[cfg(feature = "aes-ni")]
pub use aes_ctr::AesGcmStream;
