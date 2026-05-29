//! Split FFI — Dedicated XOR Key-Splitter for Homomorphic Encryption
//!
//! Split operates on a 2048-byte seed and provides key splitting:
//! - `kelvin_split_new` / `kelvin_split_free` — lifecycle
//! - `kelvin_split_generate_master_key` — generate master key K
//! - `kelvin_split_key` — split into (A, B) where A xor B = K

use std::ffi::CString;
use std::os::raw::c_char;

use kelvin::{KelvinSplit, PHOTON_BASE_SEED_SIZE};

use crate::c_api::set_error;

/// Opaque handle to a KelvinSplit context.
#[derive(Debug)]
pub struct SplitCtx {
    inner: KelvinSplit,
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_new(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    error_out: *mut *mut c_char,
) -> *mut SplitCtx {
    if seed.is_null() || seed_len != PHOTON_BASE_SEED_SIZE {
        set_error(error_out, "seed must be exactly 2048 bytes");
        return std::ptr::null_mut();
    }
    let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let inner = KelvinSplit::new(seed_arr, max_reseeds);
    Box::into_raw(Box::new(SplitCtx { inner }))
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_generate_master_key(
    ctx: *mut SplitCtx,
    key_out: *mut u8,
    key_len: usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    match ctx.inner.generate_master_key(key_len) {
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
pub unsafe extern "C" fn kelvin_split_key(
    ctx: *mut SplitCtx,
    key_len: usize,
    a_out: *mut u8,
    b_out: *mut u8,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    match ctx.inner.split_key(key_len) {
        Ok((a, b)) => {
            unsafe {
                std::ptr::copy_nonoverlapping(a.as_ptr(), a_out, key_len);
                std::ptr::copy_nonoverlapping(b.as_ptr(), b_out, key_len);
            }
            0
        },
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_encrypt(
    ctx: *mut SplitCtx,
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

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_decrypt(
    ctx: *mut SplitCtx,
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

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_free(ctx: *mut SplitCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_split_free_string(s: *mut c_char) {
    if !s.is_null() {
        drop(unsafe { CString::from_raw(s) });
    }
}
