//! Prism FFI — Standalone OTP Key Generator for Homomorphic Encryption
//!
//! Prism operates on a 2048-byte seed (same as Photon) and provides:
//! - `kelvin_prism_new` / `kelvin_prism_free` — lifecycle
//! - `kelvin_prism_generate_otp_key` — generate OTP key material
//! - `kelvin_prism_split_key` — split a key into two pads
//! - `kelvin_prism_encrypt` / `kelvin_prism_decrypt` — domain-separated XOR

use std::ffi::CString;
use std::os::raw::c_char;

use kelvin::{KelvinPrism, PHOTON_BASE_SEED_SIZE};

use crate::c_api::set_error;

/// Opaque handle to a KelvinPrism context.
#[derive(Debug)]
pub struct PrismCtx {
    inner: KelvinPrism,
}

/// Create a new Prism context from a seed.
///
/// Returns a pointer to the context, or null on error.
///
/// # Safety
///
/// - `seed` must point to `seed_len` bytes (must be exactly 2048).
/// - `error_out` must be a valid pointer to a `*mut c_char`.
/// - The returned context must be freed with `kelvin_prism_free`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_new(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    error_out: *mut *mut c_char,
) -> *mut PrismCtx {
    if seed.is_null() || seed_len != PHOTON_BASE_SEED_SIZE {
        set_error(error_out, "seed must be exactly 2048 bytes");
        return std::ptr::null_mut();
    }
    let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let inner = KelvinPrism::new(seed_arr, max_reseeds);
    Box::into_raw(Box::new(PrismCtx { inner }))
}

/// Generate an OTP key of `key_len` bytes.
///
/// Writes `key_len` bytes to `key_out`. Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_prism_new`.
/// - `key_out` must point to at least `key_len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_generate_otp_key(
    ctx: *mut PrismCtx,
    key_out: *mut u8,
    key_len: usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    match ctx.inner.generate_otp_key(key_len) {
        Ok(key) => {
            unsafe {
                std::ptr::copy_nonoverlapping(key.as_ptr(), key_out, key.len());
            }
            0
        },
        Err(_) => -1,
    }
}

/// Split a key into two pads (A, B) where A xor B = K.
///
/// Writes `key_len` bytes to `a_out` and `b_out`. Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_prism_new`.
/// - `a_out` and `b_out` must each point to at least `key_len` writable bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_split_key(
    ctx: *mut PrismCtx,
    key_len: usize,
    a_out: *mut u8,
    b_out: *mut u8,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    let len = key_len;
    match ctx.inner.split_key(len) {
        Ok((a, b)) => {
            unsafe {
                std::ptr::copy_nonoverlapping(a.as_ptr(), a_out, len);
                std::ptr::copy_nonoverlapping(b.as_ptr(), b_out, len);
            }
            0
        },
        Err(_) => -1,
    }
}

/// Encrypt data in-place using Prism's domain-separated keystream.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_prism_new`.
/// - `data` must point to at least `len` readable/writable bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_encrypt(
    ctx: *mut PrismCtx,
    data: *mut u8,
    len: usize,
) -> i32 {
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

/// Decrypt data in-place using Prism's domain-separated keystream.
///
/// Returns 0 on success, -1 on error.
///
/// # Safety
///
/// - `ctx` must be a valid pointer from `kelvin_prism_new`.
/// - `data` must point to at least `len` readable/writable bytes.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_decrypt(
    ctx: *mut PrismCtx,
    data: *mut u8,
    len: usize,
) -> i32 {
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

/// Free a Prism context created with `kelvin_prism_new`.
///
/// # Safety
///
/// `ctx` must be a valid pointer from `kelvin_prism_new`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_free(ctx: *mut PrismCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

/// Free a C string allocated by a Prism function.
///
/// # Safety
///
/// `s` must be a pointer returned by a Kelvin function.
#[no_mangle]
pub unsafe extern "C" fn kelvin_prism_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}
