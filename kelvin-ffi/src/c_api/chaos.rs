//! Chaos (V2) Streaming FFI

use std::ffi::CStr;
use std::os::raw::c_char;

use kelvin::OrbitalConfig;
use kelvin::streaming::{ChaosDecryptor, ChaosEncryptor};
use kelvin_core::IntegrationMethod;

use crate::c_api::{
    map_error, set_error,
    streaming::{
        streaming_decrypt_finalize_impl, streaming_decrypt_update_impl,
        streaming_finalize_impl, streaming_update_impl, ChaosDecryptorCtx,
        ChaosEncryptorCtx,
    },
};

/// Create a new Chaos streaming encryptor from a JSON config string.
///
/// # Safety
///
/// - `config_json` must be a valid null-terminated C string.
/// - `error_out` must be a valid pointer to a `*mut c_char`.
/// - The returned context must be freed with `kelvin_chaos_encryptor_free`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_encryptor_new(
    config_json: *const c_char,
    bytes_per_step: u64,
    error_out: *mut *mut c_char,
) -> *mut ChaosEncryptorCtx {
    if config_json.is_null() {
        set_error(error_out, "config_json must not be null");
        return std::ptr::null_mut();
    }
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
    match ChaosEncryptor::new(config, bytes_per_step) {
        Ok(inner) => Box::into_raw(Box::new(ChaosEncryptorCtx { inner })),
        Err(e) => {
            map_error(error_out, e);
            std::ptr::null_mut()
        },
    }
}

/// Create a new Chaos streaming encryptor with a configurable integration method.
///
/// `method`: 0 = Verlet (default), 1 = Euler.
///
/// # Safety
///
/// Same as `kelvin_chaos_encryptor_new`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_encryptor_new_with_method(
    config_json: *const c_char,
    bytes_per_step: u64,
    method: i32,
    error_out: *mut *mut c_char,
) -> *mut ChaosEncryptorCtx {
    if config_json.is_null() {
        set_error(error_out, "config_json must not be null");
        return std::ptr::null_mut();
    }
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
    let integration_method =
        if method == 1 { IntegrationMethod::Euler } else { IntegrationMethod::Verlet };
    match ChaosEncryptor::new_with_method(config, bytes_per_step, integration_method) {
        Ok(inner) => Box::into_raw(Box::new(ChaosEncryptorCtx { inner })),
        Err(e) => {
            map_error(error_out, e);
            std::ptr::null_mut()
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_encryptor_update(
    ctx: *mut ChaosEncryptorCtx,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    output_len: *mut usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    streaming_update_impl(&mut ctx.inner, input, input_len, output, output_cap, output_len)
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_encryptor_finalize(
    ctx: *mut ChaosEncryptorCtx,
    tag_out: *mut u8,
    tag_cap: usize,
    tag_len: *mut usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    streaming_finalize_impl(&mut ctx.inner, tag_out, tag_cap, tag_len)
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_encryptor_free(ctx: *mut ChaosEncryptorCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

/// Create a new Chaos streaming decryptor from a JSON config string.
///
/// # Safety
///
/// Same as `kelvin_chaos_encryptor_new`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_decryptor_new(
    config_json: *const c_char,
    bytes_per_step: u64,
    error_out: *mut *mut c_char,
) -> *mut ChaosDecryptorCtx {
    if config_json.is_null() {
        set_error(error_out, "config_json must not be null");
        return std::ptr::null_mut();
    }
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
    match ChaosDecryptor::new(config, bytes_per_step) {
        Ok(inner) => Box::into_raw(Box::new(ChaosDecryptorCtx { inner })),
        Err(e) => {
            map_error(error_out, e);
            std::ptr::null_mut()
        },
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_decryptor_update(
    ctx: *mut ChaosDecryptorCtx,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    output_len: *mut usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    streaming_decrypt_update_impl(&mut ctx.inner, input, input_len, output, output_cap, output_len)
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_decryptor_finalize(
    ctx: *mut ChaosDecryptorCtx,
    tag: *const u8,
    tag_len: usize,
) -> i32 {
    let ctx = match unsafe { ctx.as_mut() } {
        Some(c) => c,
        None => return -1,
    };
    streaming_decrypt_finalize_impl(&mut ctx.inner, tag, tag_len)
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_chaos_decryptor_free(ctx: *mut ChaosDecryptorCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}
