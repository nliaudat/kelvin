//! C FFI bindings for the Kelvin cryptosystem.
//!
//! Provides a C ABI for integration with iOS, Android, and embedded systems.

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use kelvin::{Kelvin, OrbitalConfig};

/// Opaque handle to a Kelvin context.
#[derive(Debug)]
pub struct KelvinCtx {
    inner: Kelvin,
}

/// Create a new Kelvin context from a JSON config string.
///
/// Returns a pointer to the context, or null on error.
/// On error, `error_out` is set to a string describing the error.
///
/// # Safety
///
/// - `config_json` must be a valid null-terminated C string.
/// - `error_out` must be a valid pointer to a `*mut c_char`.
/// - The returned context must be freed with `kelvin_free`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_new(
    config_json: *const c_char,
    error_out: *mut *mut c_char,
) -> *mut KelvinCtx {
    let config_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid UTF-8: {}", e));
            return std::ptr::null_mut();
        },
    };

    let config = match OrbitalConfig::from_json(config_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(error_out, &format!("config parse error: {}", e));
            return std::ptr::null_mut();
        },
    };

    let kelvin = match Kelvin::new(config) {
        Ok(k) => k,
        Err(e) => {
            set_error(error_out, &format!("kelvin init error: {}", e));
            return std::ptr::null_mut();
        },
    };

    Box::into_raw(Box::new(KelvinCtx { inner: kelvin }))
}

/// Encrypt data in-place using AEAD.
///
/// The buffer must have a total size of `len` bytes (minimum 16). The plaintext
/// occupies the first `len - 16` bytes; the AEAD authentication tag
/// (Poly1305 or GMAC) is written at `data[len-16..len]`.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_new`.
/// - `data` must point to a buffer of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_encrypt(ctx: *mut KelvinCtx, data: *mut u8, len: usize) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    let slice = unsafe { std::slice::from_raw_parts_mut(data, len) };
    match ctx.inner.encrypt(slice) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Decrypt data in-place using AEAD.
///
/// The buffer must contain ciphertext + 16-byte authentication tag.
/// The buffer must have a total size of `len` bytes (minimum 16).
/// The first `len - 16` bytes are the ciphertext; the last 16 bytes
/// are the tag. On success, the first `len - 16` bytes contain the
/// recovered plaintext.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_new`.
/// - `data` must point to a buffer of at least `len` bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_decrypt(ctx: *mut KelvinCtx, data: *mut u8, len: usize) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    let slice = unsafe { std::slice::from_raw_parts_mut(data, len) };
    match ctx.inner.decrypt(slice) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

/// Get remaining safe bytes.
///
/// # Safety
///
/// `ctx` must be a valid pointer from `kelvin_new`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_remaining_bytes(ctx: *const KelvinCtx) -> u64 {
    match unsafe { ctx.as_ref() } {
        Some(c) => c.inner.remaining_safe_bytes(),
        None => 0,
    }
}

/// Free a Kelvin context created with `kelvin_new`.
///
/// # Safety
///
/// `ctx` must be a valid pointer from `kelvin_new` that has not been freed yet.
#[no_mangle]
pub unsafe extern "C" fn kelvin_free(ctx: *mut KelvinCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

/// Free a C string allocated by the Kelvin library (e.g., error messages).
///
/// # Safety
///
/// `s` must be a valid pointer returned by a Kelvin function that allocates
/// a C string, and must not have been freed yet.
#[no_mangle]
pub unsafe extern "C" fn kelvin_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}

/// Set an error string in the output pointer.
///
/// # Safety
///
/// `error_out` must be a valid pointer to a `*mut c_char`.
unsafe fn set_error(error_out: *mut *mut c_char, msg: &str) {
    if !error_out.is_null() {
        let c_str = CString::new(msg).unwrap_or_default();
        unsafe {
            *error_out = c_str.into_raw();
        }
    }
}
