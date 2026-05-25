//! C FFI bindings for the Kelvin cryptosystem.
//!
//! Provides a C ABI for integration with iOS, Android, and embedded systems.
//!
//! ## Streaming API
//!
//! All 4 modes (Photon, Quantum, Chaos, Secure) expose streaming encrypt/decrypt
//! via the standard lifecycle:
//!
//! ```c
//! // Create
//! PhotonEncryptorCtx* ctx = kelvin_photon_encryptor_new(seed, 2048, 1000, &err);
//!
//! // Update (call multiple times)
//! uint8_t output[256];
//! size_t output_len;
//! kelvin_photon_encryptor_update(ctx, input, input_len, output, sizeof(output), &output_len);
//!
//! // Finalize
//! uint8_t tag[32];
//! size_t tag_len;
//! kelvin_photon_encryptor_finalize(ctx, tag, sizeof(tag), &tag_len);
//!
//! // Free
//! kelvin_photon_encryptor_free(ctx);
//! ```

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

use kelvin::streaming::{
    decrypt_file_streaming, encrypt_file_streaming, ChaosDecryptor, ChaosEncryptor,
    PhotonDecryptor, PhotonEncryptor, QuantumDecryptor, QuantumEncryptor, SecureDecryptor,
    SecureEncryptor, StreamDecrypt, StreamEncrypt,
};
use kelvin::{Kelvin, KelvinError, OrbitalConfig, PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE};
use kelvin_core::IntegrationMethod;

// ============================================================================
// Error helpers
// ============================================================================

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

/// Convert a KelvinError to a C error string and return -1.
fn map_error(error_out: *mut *mut c_char, e: KelvinError) -> i32 {
    unsafe { set_error(error_out, &format!("KelvinError: {}", e)) };
    -1
}

// ============================================================================
// V1 Kelvin — AEAD encrypt/decrypt (existing)
// ============================================================================

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

// ============================================================================
// Streaming API — Opaque handles
// ============================================================================

macro_rules! define_streaming_ctx {
    ($name:ident, $inner:ty) => {
        #[derive(Debug)]
        pub struct $name {
            inner: $inner,
        }
    };
}

define_streaming_ctx!(PhotonEncryptorCtx, PhotonEncryptor);
define_streaming_ctx!(PhotonDecryptorCtx, PhotonDecryptor);
define_streaming_ctx!(QuantumEncryptorCtx, QuantumEncryptor);
define_streaming_ctx!(QuantumDecryptorCtx, QuantumDecryptor);
define_streaming_ctx!(ChaosEncryptorCtx, ChaosEncryptor);
define_streaming_ctx!(ChaosDecryptorCtx, ChaosDecryptor);
define_streaming_ctx!(SecureEncryptorCtx, SecureEncryptor);
define_streaming_ctx!(SecureDecryptorCtx, SecureDecryptor);

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
unsafe fn streaming_update_impl(
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
unsafe fn streaming_decrypt_update_impl(
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
unsafe fn streaming_finalize_impl(
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
unsafe fn streaming_decrypt_finalize_impl(
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

macro_rules! gen_stream_encrypt_ffi {
    ($prefix:ident, $ctx_type:ty, $inner_type:ty, $seed_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _encryptor_new>](
                seed: *const u8,
                seed_len: usize,
                max_reseeds: u64,
                error_out: *mut *mut c_char,
            ) -> *mut $ctx_type {
                if seed.is_null() || seed_len != std::mem::size_of::<$seed_type>() {
                    set_error(error_out, "invalid seed");
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
                streaming_update_impl(&mut ctx.inner, input, input_len, output, output_cap, output_len)
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
                streaming_finalize_impl(&mut ctx.inner, tag_out, tag_cap, tag_len)
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

macro_rules! gen_stream_decrypt_ffi {
    ($prefix:ident, $ctx_type:ty, $inner_type:ty, $seed_type:ty) => {
        paste::paste! {
            #[no_mangle]
            pub unsafe extern "C" fn [<kelvin_ $prefix _decryptor_new>](
                seed: *const u8,
                seed_len: usize,
                max_reseeds: u64,
                error_out: *mut *mut c_char,
            ) -> *mut $ctx_type {
                if seed.is_null() || seed_len != std::mem::size_of::<$seed_type>() {
                    set_error(error_out, "invalid seed");
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
                streaming_decrypt_update_impl(&mut ctx.inner, input, input_len, output, output_cap, output_len)
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
                streaming_decrypt_finalize_impl(&mut ctx.inner, tag, tag_len)
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

// ============================================================================
// Photon (V3) Streaming FFI
// ============================================================================

gen_stream_encrypt_ffi!(photon, PhotonEncryptorCtx, PhotonEncryptor, [u8; PHOTON_BASE_SEED_SIZE]);
gen_stream_decrypt_ffi!(photon, PhotonDecryptorCtx, PhotonDecryptor, [u8; PHOTON_BASE_SEED_SIZE]);

// ============================================================================
// Quantum (H) Streaming FFI
// ============================================================================

gen_stream_encrypt_ffi!(
    quantum,
    QuantumEncryptorCtx,
    QuantumEncryptor,
    [u8; QUANTUM_BASE_SEED_SIZE]
);
gen_stream_decrypt_ffi!(
    quantum,
    QuantumDecryptorCtx,
    QuantumDecryptor,
    [u8; QUANTUM_BASE_SEED_SIZE]
);

// ============================================================================
// Chaos (V2) Streaming FFI
// ============================================================================

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

// ============================================================================
// Secure (V1) Streaming FFI
// ============================================================================

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

// ============================================================================
// File-level streaming convenience functions
// ============================================================================

// Photon file streaming

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_photon_encrypt(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if seed.is_null() || seed_len != std::mem::size_of::<[u8; PHOTON_BASE_SEED_SIZE]>() {
        set_error(error_out, "invalid seed");
        return -1;
    }
    let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let encryptor = PhotonEncryptor::new(seed_arr, max_reseeds);
    match encrypt_file_streaming(
        encryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_photon_decrypt(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    tag_len: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if seed.is_null() || seed_len != std::mem::size_of::<[u8; PHOTON_BASE_SEED_SIZE]>() {
        set_error(error_out, "invalid seed");
        return -1;
    }
    let mut seed_arr = [0u8; PHOTON_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let decryptor = PhotonDecryptor::new(seed_arr, max_reseeds);
    match decrypt_file_streaming(
        decryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
        tag_len,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

// Quantum file streaming

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_quantum_encrypt(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if seed.is_null() || seed_len != std::mem::size_of::<[u8; QUANTUM_BASE_SEED_SIZE]>() {
        set_error(error_out, "invalid seed");
        return -1;
    }
    let mut seed_arr = [0u8; QUANTUM_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let encryptor = QuantumEncryptor::new(seed_arr, max_reseeds);
    match encrypt_file_streaming(
        encryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_quantum_decrypt(
    seed: *const u8,
    seed_len: usize,
    max_reseeds: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    tag_len: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if seed.is_null() || seed_len != std::mem::size_of::<[u8; QUANTUM_BASE_SEED_SIZE]>() {
        set_error(error_out, "invalid seed");
        return -1;
    }
    let mut seed_arr = [0u8; QUANTUM_BASE_SEED_SIZE];
    unsafe {
        std::ptr::copy_nonoverlapping(seed, seed_arr.as_mut_ptr(), seed_arr.len());
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let decryptor = QuantumDecryptor::new(seed_arr, max_reseeds);
    match decrypt_file_streaming(
        decryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
        tag_len,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

// Chaos file streaming
#[no_mangle]
pub unsafe extern "C" fn kelvin_file_chaos_encrypt(
    config_json: *const c_char,
    bytes_per_step: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    let config_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid UTF-8: {}", e));
            return -1;
        },
    };
    let config = match OrbitalConfig::from_json(config_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(error_out, &format!("config parse error: {}", e));
            return -1;
        },
    };
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let encryptor = match ChaosEncryptor::new(config, bytes_per_step) {
        Ok(e) => e,
        Err(e) => return map_error(error_out, e),
    };
    match encrypt_file_streaming(
        encryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_chaos_decrypt(
    config_json: *const c_char,
    bytes_per_step: u64,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    tag_len: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    let config_str = match unsafe { CStr::from_ptr(config_json) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid UTF-8: {}", e));
            return -1;
        },
    };
    let config = match OrbitalConfig::from_json(config_str) {
        Ok(c) => c,
        Err(e) => {
            set_error(error_out, &format!("config parse error: {}", e));
            return -1;
        },
    };
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let decryptor = match ChaosDecryptor::new(config, bytes_per_step) {
        Ok(d) => d,
        Err(e) => return map_error(error_out, e),
    };
    match decrypt_file_streaming(
        decryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
        tag_len,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

// Secure file streaming
#[no_mangle]
pub unsafe extern "C" fn kelvin_file_secure_encrypt(
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if key.is_null() || key_len != 32 {
        set_error(error_out, "key must be 32 bytes");
        return -1;
    }
    if nonce.is_null() || nonce_len != 12 {
        set_error(error_out, "nonce must be 12 bytes");
        return -1;
    }
    let mut key_arr = [0u8; 32];
    let mut nonce_arr = [0u8; 12];
    unsafe {
        std::ptr::copy_nonoverlapping(key, key_arr.as_mut_ptr(), 32);
        std::ptr::copy_nonoverlapping(nonce, nonce_arr.as_mut_ptr(), 12);
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let encryptor = SecureEncryptor::new(key_arr, nonce_arr);
    match encrypt_file_streaming(
        encryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}

#[no_mangle]
pub unsafe extern "C" fn kelvin_file_secure_decrypt(
    key: *const u8,
    key_len: usize,
    nonce: *const u8,
    nonce_len: usize,
    input_path: *const c_char,
    output_path: *const c_char,
    buffer_size: usize,
    tag_len: usize,
    error_out: *mut *mut c_char,
) -> i32 {
    if key.is_null() || key_len != 32 {
        set_error(error_out, "key must be 32 bytes");
        return -1;
    }
    if nonce.is_null() || nonce_len != 12 {
        set_error(error_out, "nonce must be 12 bytes");
        return -1;
    }
    let mut key_arr = [0u8; 32];
    let mut nonce_arr = [0u8; 12];
    unsafe {
        std::ptr::copy_nonoverlapping(key, key_arr.as_mut_ptr(), 32);
        std::ptr::copy_nonoverlapping(nonce, nonce_arr.as_mut_ptr(), 12);
    }
    let input = match unsafe { CStr::from_ptr(input_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid input path: {}", e));
            return -1;
        },
    };
    let output = match unsafe { CStr::from_ptr(output_path) }.to_str() {
        Ok(s) => s,
        Err(e) => {
            set_error(error_out, &format!("invalid output path: {}", e));
            return -1;
        },
    };
    let decryptor = SecureDecryptor::new(key_arr, nonce_arr);
    match decrypt_file_streaming(
        decryptor,
        std::path::Path::new(input),
        std::path::Path::new(output),
        buffer_size,
        tag_len,
    ) {
        Ok(()) => 0,
        Err(e) => map_error(error_out, e),
    }
}
