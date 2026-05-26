//! Photon (V3) Streaming FFI

use kelvin::PHOTON_BASE_SEED_SIZE;

use crate::c_api::streaming::{PhotonDecryptorCtx, PhotonEncryptorCtx};
use crate::gen_stream_decrypt_ffi;
use crate::gen_stream_encrypt_ffi;

gen_stream_encrypt_ffi!(
    photon,
    PhotonEncryptorCtx,
    kelvin::streaming::PhotonEncryptor,
    [u8; PHOTON_BASE_SEED_SIZE]
);
gen_stream_decrypt_ffi!(
    photon,
    PhotonDecryptorCtx,
    kelvin::streaming::PhotonDecryptor,
    [u8; PHOTON_BASE_SEED_SIZE]
);
