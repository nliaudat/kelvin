//! Quantum (H) Streaming FFI

use kelvin::QUANTUM_BASE_SEED_SIZE;

use crate::c_api::streaming::{QuantumDecryptorCtx, QuantumEncryptorCtx};
use crate::gen_stream_decrypt_ffi;
use crate::gen_stream_encrypt_ffi;

gen_stream_encrypt_ffi!(
    quantum,
    QuantumEncryptorCtx,
    kelvin::streaming::QuantumEncryptor,
    [u8; QUANTUM_BASE_SEED_SIZE]
);
gen_stream_decrypt_ffi!(
    quantum,
    QuantumDecryptorCtx,
    kelvin::streaming::QuantumDecryptor,
    [u8; QUANTUM_BASE_SEED_SIZE]
);
