//! File-level streaming convenience functions.
//!
//! Provides `kelvin_file_*` functions for all 4 modes (Photon, Quantum, Chaos, Secure)
//! that encrypt/decrypt entire files using the streaming API.

use std::ffi::CStr;
use std::os::raw::c_char;

use kelvin::streaming::{
    decrypt_file_streaming, encrypt_file_streaming, ChaosDecryptor, ChaosEncryptor,
    PhotonDecryptor, PhotonEncryptor, QuantumDecryptor, QuantumEncryptor, SecureDecryptor,
    SecureEncryptor,
};
use kelvin::{OrbitalConfig, PHOTON_BASE_SEED_SIZE, QUANTUM_BASE_SEED_SIZE};

use crate::c_api::{map_error, set_error};

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
