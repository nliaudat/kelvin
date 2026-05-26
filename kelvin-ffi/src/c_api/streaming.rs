//! Streaming helpers and macros for C FFI.
//!
//! Provides the shared streaming update/finalize implementations and the
//! `define_streaming_ctx!`, `gen_stream_encrypt_ffi!`, and
//! `gen_stream_decrypt_ffi!` macros used by the Photon and Quantum modules.

use kelvin::streaming::{StreamDecrypt, StreamEncrypt};

// ============================================================================
// Streaming API — Opaque handles
// ============================================================================

/// Macro to define a streaming context struct with a public `inner` field.
macro_rules! define_streaming_ctx {
    ($name:ident, $inner:ty) => {
        #[derive(Debug)]
        #[allow(dead_code)]
        pub struct $name {
            pub(crate) inner: $inner,
        }
    };
}

// The macro is used below in this file; the re-export is for other modules.
#[allow(unused_imports)]
pub(crate) use define_streaming_ctx;

define_streaming_ctx!(PhotonEncryptorCtx, kelvin::streaming::PhotonEncryptor);
define_streaming_ctx!(PhotonDecryptorCtx, kelvin::streaming::PhotonDecryptor);
define_streaming_ctx!(QuantumEncryptorCtx, kelvin::streaming::QuantumEncryptor);
define_streaming_ctx!(QuantumDecryptorCtx, kelvin::streaming::QuantumDecryptor);
define_streaming_ctx!(ChaosEncryptorCtx, kelvin::streaming::ChaosEncryptor);
define_streaming_ctx!(ChaosDecryptorCtx, kelvin::streaming::ChaosDecryptor);
define_streaming_ctx!(SecureEncryptorCtx, kelvin::streaming::SecureEncryptor);
define_streaming_ctx!(SecureDecryptorCtx, kelvin::streaming::SecureDecryptor);

// ============================================================================
// Helper: streaming update/finalize C functions
// ============================================================================

/// Helper to implement the streaming update pattern.
///
/// # Safety
///
/// - `ctx` must be a valid non-null pointer.
/// - `input` must point to `input_len` readable bytes (may be null if input_len == 0).
/// - `output` must point to at least `output_cap` writable bytes (may be null if output_cap == 0).
/// - `output_len` must be a valid non-null pointer to a `size_t`.
pub(crate) unsafe fn streaming_update_impl(
    ctx: &mut dyn StreamEncrypt,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    output_len: *mut usize,
) -> i32 {
    if output_len.is_null() {
        return -1;
    }
    let input_slice = if input_len == 0 || input.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_len) }
    };
    let mut out = Vec::new();
    match ctx.update(input_slice, &mut out) {
        Ok(()) => {
            if out.len() > output_cap {
                return -1; // buffer too small
            }
            if !out.is_empty() {
                if output.is_null() {
                    return -1;
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(out.as_ptr(), output, out.len());
                }
            }
            unsafe {
                *output_len = out.len();
            }
            0
        },
        Err(_) => -1,
    }
}

/// Helper to implement the streaming decrypt update pattern.
///
/// # Safety
///
/// Same as `streaming_update_impl`.
pub(crate) unsafe fn streaming_decrypt_update_impl(
    ctx: &mut dyn StreamDecrypt,
    input: *const u8,
    input_len: usize,
    output: *mut u8,
    output_cap: usize,
    output_len: *mut usize,
) -> i32 {
    if output_len.is_null() {
        return -1;
    }
    let input_slice = if input_len == 0 || input.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(input, input_len) }
    };
    let mut out = Vec::new();
    match ctx.update(input_slice, &mut out) {
        Ok(()) => {
            if out.len() > output_cap {
                return -1;
            }
            if !out.is_empty() {
                if output.is_null() {
                    return -1;
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(out.as_ptr(), output, out.len());
                }
            }
            unsafe {
                *output_len = out.len();
            }
            0
        },
        Err(_) => -1,
    }
}

/// Helper to implement the streaming finalize pattern.
///
/// # Safety
///
/// - `ctx` must be a valid non-null pointer.
/// - `tag_out` must point to at least `tag_cap` writable bytes (may be null if tag_cap == 0).
/// - `tag_len` must be a valid non-null pointer to a `size_t`.
pub(crate) unsafe fn streaming_finalize_impl(
    ctx: &mut dyn StreamEncrypt,
    tag_out: *mut u8,
    tag_cap: usize,
    tag_len: *mut usize,
) -> i32 {
    if tag_len.is_null() {
        return -1;
    }
    match ctx.finalize() {
        Ok(tag) => {
            if tag.len() > tag_cap {
                return -1;
            }
            if !tag.is_empty() {
                if tag_out.is_null() {
                    return -1;
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(tag.as_ptr(), tag_out, tag.len());
                }
            }
            unsafe {
                *tag_len = tag.len();
            }
            0
        },
        Err(_) => -1,
    }
}

/// Helper to implement the streaming decrypt finalize pattern.
///
/// # Safety
///
/// - `ctx` must be a valid non-null pointer.
/// - `tag` must point to `tag_len` readable bytes (may be null if tag_len == 0).
pub(crate) unsafe fn streaming_decrypt_finalize_impl(
    ctx: &mut dyn StreamDecrypt,
    tag: *const u8,
    tag_len: usize,
) -> i32 {
    let tag_slice = if tag_len == 0 || tag.is_null() {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(tag, tag_len) }
    };
    match ctx.finalize(tag_slice) {
        Ok(()) => 0,
        Err(_) => -1,
    }
}

// ============================================================================
// Macro: generate C FFI functions for a streaming encryptor
// ============================================================================

/// Generate C FFI functions for a streaming encryptor.
///
/// Generates: `kelvin_{prefix}_encryptor_new`, `kelvin_{prefix}_encryptor_update`,
/// `kelvin_{prefix}_encryptor_finalize`, `kelvin_{prefix}_encryptor_free`.
///
/// The macro uses `$crate::c_api::streaming::*` paths so it works from any submodule.
#[macro_export]
macro_rules! gen_stream_encrypt_ffi {
    ($prefix:ident, $ctx_type:ty, $inner_type:ty, $seed_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _encryptor_new>](
                seed: *const u8,
                seed_len: usize,
                max_reseeds: u64,
                error_out: *mut *mut std::os::raw::c_char,
            ) -> *mut $ctx_type {
                if seed.is_null() || seed_len != std::mem::size_of::<$seed_type>() {
                    $crate::c_api::set_error(error_out, "invalid seed");
                    return std::ptr::null_mut();
                }
                let mut seed_arr = [0u8; std::mem::size_of::<$seed_type>()];
                unsafe {
                    std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
                }
                let inner = <$inner_type>::new(seed_arr, max_reseeds);
                Box::into_raw(Box::new($ctx_type { inner }))
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _encryptor_update>](
                ctx: *mut $ctx_type,
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
                $crate::c_api::streaming::streaming_update_impl(
                    &mut ctx.inner, input, input_len, output, output_cap, output_len,
                )
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _encryptor_finalize>](
                ctx: *mut $ctx_type,
                tag_out: *mut u8,
                tag_cap: usize,
                tag_len: *mut usize,
            ) -> i32 {
                let ctx = match unsafe { ctx.as_mut() } {
                    Some(c) => c,
                    None => return -1,
                };
                $crate::c_api::streaming::streaming_finalize_impl(
                    &mut ctx.inner, tag_out, tag_cap, tag_len,
                )
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _encryptor_free>](
                ctx: *mut $ctx_type,
            ) {
                if !ctx.is_null() {
                    drop(unsafe { Box::from_raw(ctx) });
                }
            }
        }
    };
}

/// Generate C FFI functions for a streaming decryptor.
///
/// Generates: `kelvin_{prefix}_decryptor_new`, `kelvin_{prefix}_decryptor_update`,
/// `kelvin_{prefix}_decryptor_finalize`, `kelvin_{prefix}_decryptor_free`.
///
/// The macro uses `$crate::c_api::streaming::*` paths so it works from any submodule.
#[macro_export]
macro_rules! gen_stream_decrypt_ffi {
    ($prefix:ident, $ctx_type:ty, $inner_type:ty, $seed_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _decryptor_new>](
                seed: *const u8,
                seed_len: usize,
                max_reseeds: u64,
                error_out: *mut *mut std::os::raw::c_char,
            ) -> *mut $ctx_type {
                if seed.is_null() || seed_len != std::mem::size_of::<$seed_type>() {
                    $crate::c_api::set_error(error_out, "invalid seed");
                    return std::ptr::null_mut();
                }
                let mut seed_arr = [0u8; std::mem::size_of::<$seed_type>()];
                unsafe {
                    std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
                }
                let inner = <$inner_type>::new(seed_arr, max_reseeds);
                Box::into_raw(Box::new($ctx_type { inner }))
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _decryptor_update>](
                ctx: *mut $ctx_type,
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
                $crate::c_api::streaming::streaming_decrypt_update_impl(
                    &mut ctx.inner, input, input_len, output, output_cap, output_len,
                )
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _decryptor_finalize>](
                ctx: *mut $ctx_type,
                tag: *const u8,
                tag_len: usize,
            ) -> i32 {
                let ctx = match unsafe { ctx.as_mut() } {
                    Some(c) => c,
                    None => return -1,
                };
                $crate::c_api::streaming::streaming_decrypt_finalize_impl(
                    &mut ctx.inner, tag, tag_len,
                )
            }

            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _decryptor_free>](
                ctx: *mut $ctx_type,
            ) {
                if !ctx.is_null() {
                    drop(unsafe { Box::from_raw(ctx) });
                }
            }
        }
    };
}
