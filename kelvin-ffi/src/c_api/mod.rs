//! C FFI bindings for the Kelvin cryptosystem.
//!
//! Provides a C ABI for integration with iOS, Android, and embedded systems.
//!
//! ## Modules
//!
//! - [`v1`] — V1 AEAD encrypt/decrypt (legacy)
//! - [`streaming`] — Streaming helpers and macros
//! - [`photon`] — Photon streaming encrypt/decrypt
//! - [`quantum`] — Quantum streaming encrypt/decrypt
//! - [`chaos`] — Chaos streaming encrypt/decrypt
//! - [`secure`] — Secure streaming encrypt/decrypt
//! - [`file`] — File-level streaming convenience functions

use std::ffi::CString;
use std::os::raw::c_char;

use kelvin::KelvinError;

pub mod file;
pub mod flare;
pub mod photon;
pub mod prism;
pub mod quantum;
pub mod secure;
pub mod split;
pub mod streaming;
pub mod v1;

// Re-export all public items from submodules
pub use file::*;
pub use flare::*;
pub use photon::*;
pub use prism::*;
pub use quantum::*;
pub use secure::*;
pub use split::*;
pub use streaming::*;
pub use v1::*;

// ============================================================================
// Error helpers
// ============================================================================

/// Set an error string in the output pointer.
///
/// # Safety
///
/// `error_out` must be a valid pointer to a `*mut c_char`.
pub(crate) unsafe fn set_error(error_out: *mut *mut c_char, msg: &str) {
    if !error_out.is_null() {
        let c_str = CString::new(msg).unwrap_or_default();
        unsafe {
            *error_out = c_str.into_raw();
        }
    }
}

/// Convert a KelvinError to a C error string and return -1.
pub(crate) fn map_error(error_out: *mut *mut c_char, e: KelvinError) -> i32 {
    unsafe { set_error(error_out, &format!("KelvinError: {}", e)) };
    -1
}
