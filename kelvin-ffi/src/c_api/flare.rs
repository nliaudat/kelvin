//! Flare FFI — Chaotic FHE Secret Key Generator
//!
//! Flare operates on a 2048-byte seed and provides FHE key generation:
//! - `kelvin_flare_new` / `kelvin_flare_free` — lifecycle
//! - `kelvin_flare_generate_secret_key` — raw secret key
//! - `kelvin_flare_generate_fhe_key` — scheme-specific FHE key

use std::ffi::CString;
use std::os::raw::c_char;

use kelvin::{FlareScheme, KelvinFlare, PHOTON_BASE_SEED_SIZE};

use crate::c_api::set_error;

/// Opaque handle to a KelvinFlare context.
#[derive(Debug)]
pub struct FlareCtx {
    inner: KelvinFlare,
}

fn parse_scheme(scheme: i32) -> Option<FlareScheme> {
    match scheme {
        0 => Some(FlareScheme::Bfv),
        1 => Some(FlareScheme::Ckks),
        2 => Some(FlareScheme::Tfhe),
        _ => None,
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_flare_new(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    error_out: *mut *mut c_char,
) -> *mut FlareCtx {
    if seed.is_null() || seed_len != PHOTON_BASE_SEED_SIZE {
        set_error(error_out, "seed must be exactly 2048 bytes");
        return std::ptr::null_mut();
    }
    let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let inner = KelvinFlare::new(seed_arr, max_reseeds);
    Box::into_raw(Box::new(FlareCtx { inner }))
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_flare_generate_secret_key(
    ctx: *mut FlareCtx,
    key_out: *mut u8,
    key_len: usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    match ctx.inner.generate_secret_key(key_len) {
        Ok(key) => {
            unsafe {
                std::ptr::copy_nonoverlapping(key.as_ptr(), key_out, key.len());
            }
            0
        },
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_flare_generate_fhe_key(
    ctx: *mut FlareCtx,
    scheme: i32,
    key_out: *mut u8,
    key_len: usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    let fs = match parse_scheme(scheme) {
        Some(s) => s,
        None => return -1,
    };
    match ctx.inner.generate_fhe_key(fs, key_len) {
        Ok(key) => {
            unsafe {
                std::ptr::copy_nonoverlapping(key.key().as_ptr(), key_out, key_len);
            }
            0
        },
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_flare_free(ctx: *mut FlareCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_flare_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}
