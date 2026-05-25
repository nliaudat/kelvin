//! Secure (V1) Streaming FFI

use std::os::raw::c_char;

use kelvin::streaming::{SecureDecryptor, SecureEncryptor};

use crate::c_api::{
    set_error,
    streaming::{
        streaming_decrypt_finalize_impl, streaming_decrypt_update_impl, streaming_finalize_impl,
        streaming_update_impl, SecureDecryptorCtx, SecureEncryptorCtx,
    },
};

/// Create a new Secure streaming encryptor.
///
/// `key` must be 32 bytes, `nonce` must be 12 bytes.
///
/// # Safety
///
/// - `key` must point to 32 readable bytes.
/// - `nonce` must point to 12 readable bytes.
/// - `error_out` must be a valid pointer to a `*mut c_char`.
/// - The returned context must be freed with `kelvin_secure_encryptor_free`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_secure_encryptor_new(
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    error_out: *mut *mut c_char,
) -> *mut SecureEncryptorCtx {
    if key.is_null() || key_len != 32 {
        set_error(error_out, "key must be 32 bytes");
        return std::ptr::null_mut();
    }
    if nonce.is_null() || nonce_len != 12 {
        set_error(error_out, "nonce must be 12 bytes");
        return std::ptr::null_mut();
    }
    let mut key_arr = [0u8; 32];
    let mut nonce_arr = [0u8; 12];
    unsafe {
        std::ptr::copy_nonoverlapping(key, key_arr.as_mut_ptr(), 32);
        std::ptr::copy_nonoverlapping(nonce, nonce_arr.as_mut_ptr(), 12);
    }
    let inner = SecureEncryptor::new(key_arr, nonce_arr);
    Box::into_raw(Box::new(SecureEncryptorCtx { inner }))
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_secure_encryptor_update(
    ctx: *mut SecureEncryptorCtx,
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
pub unsafe extern "C" fn kelvin_secure_encryptor_finalize(
    ctx: *mut SecureEncryptorCtx,
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
pub unsafe extern "C" fn kelvin_secure_encryptor_free(ctx: *mut SecureEncryptorCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}

/// Create a new Secure streaming decryptor.
///
/// `key` must be 32 bytes, `nonce` must be 12 bytes.
///
/// # Safety
///
/// Same as `kelvin_secure_encryptor_new`.
#[no_mangle]
pub unsafe extern "C" fn kelvin_secure_decryptor_new(
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    error_out: *mut *mut c_char,
) -> *mut SecureDecryptorCtx {
    if key.is_null() || key_len != 32 {
        set_error(error_out, "key must be 32 bytes");
        return std::ptr::null_mut();
    }
    if nonce.is_null() || nonce_len != 12 {
        set_error(error_out, "nonce must be 12 bytes");
        return std::ptr::null_mut();
    }
    let mut key_arr = [0u8; 32];
    let mut nonce_arr = [0u8; 12];
    unsafe {
        std::ptr::copy_nonoverlapping(key, key_arr.as_mut_ptr(), 32);
        std::ptr::copy_nonoverlapping(nonce, nonce_arr.as_mut_ptr(), 12);
    }
    let inner = SecureDecryptor::new(key_arr, nonce_arr);
    Box::into_raw(Box::new(SecureDecryptorCtx { inner }))
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_secure_decryptor_update(
    ctx: *mut SecureDecryptorCtx,
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
pub unsafe extern "C" fn kelvin_secure_decryptor_finalize(
    ctx: *mut SecureDecryptorCtx,
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
pub unsafe extern "C" fn kelvin_secure_decryptor_free(ctx: *mut SecureDecryptorCtx) {
    if !ctx.is_null() {
        drop(unsafe { Box::from_raw(ctx) });
    }
}
