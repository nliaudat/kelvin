"""
Kelvin Cryptosystem — Python bindings via C FFI.

Provides both the V1 AEAD API and the streaming encrypt/decrypt API
for all 4 modes (Photon, Quantum, Chaos, Secure), plus Prism, Split,
and Flare for homomorphic encryption integration.
"""

import ctypes
import json
import os
from typing import Optional, Tuple

# Load the shared library
_lib_path = os.path.join(os.path.dirname(__file__), "..", "..", "..", "target", "debug")
if os.name == "nt":
    _lib_file = os.path.join(_lib_path, "kelvin_ffi.dll")
else:
    _lib_file = os.path.join(_lib_path, "libkelvin_ffi.so")

_lib = ctypes.CDLL(_lib_file)

# ============================================================================
# Error handling
# ============================================================================

class KelvinError(Exception):
    """Exception raised for Kelvin cryptosystem errors."""
    pass


def _check_error(res: int, error_out: ctypes.POINTER(ctypes.c_char_p)) -> None:
    """Check return value and raise KelvinError if non-zero."""
    if res != 0:
        err_str = error_out.contents.value
        if err_str:
            msg = err_str.decode("utf-8")
            _lib.kelvin_free_string(error_out.contents)
        else:
            msg = "unknown error"
        raise KelvinError(msg)


# ============================================================================
# V1 Kelvin — AEAD encrypt/decrypt (existing)
# ============================================================================

class Kelvin:
    """V1 AEAD encrypt/decrypt instance."""

    def __init__(self, config_json: str):
        _lib.kelvin_new.restype = ctypes.c_void_p
        _lib.kelvin_new.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)]

        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_new(config_json.encode("utf-8"), ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to initialize Kelvin")
        self._ctx = ctx
        self._closed = False

    def encrypt(self, data: bytearray) -> None:
        """Encrypt data in-place."""
        _lib.kelvin_encrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        _lib.kelvin_encrypt.restype = ctypes.c_int32
        buf = (ctypes.c_uint8 * len(data)).from_buffer(data)
        res = _lib.kelvin_encrypt(self._ctx, buf, len(data))
        if res != 0:
            raise KelvinError("encryption failed")

    def decrypt(self, data: bytearray) -> None:
        """Decrypt data in-place."""
        _lib.kelvin_decrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        _lib.kelvin_decrypt.restype = ctypes.c_int32
        buf = (ctypes.c_uint8 * len(data)).from_buffer(data)
        res = _lib.kelvin_decrypt(self._ctx, buf, len(data))
        if res != 0:
            raise KelvinError("decryption failed")

    def remaining_bytes(self) -> int:
        """Get remaining safe bytes."""
        _lib.kelvin_remaining_bytes.argtypes = [ctypes.c_void_p]
        _lib.kelvin_remaining_bytes.restype = ctypes.c_uint64
        return _lib.kelvin_remaining_bytes(self._ctx)

    def close(self) -> None:
        """Release resources."""
        if self._ctx:
            _lib.kelvin_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_free(self._ctx)
            self._ctx = None
            self._closed = True

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Streaming API — Generic helpers
# ============================================================================

def _streaming_update(lib_fn, ctx, input_data: bytes) -> bytes:
    """Perform a streaming update and return the output."""
    if len(input_data) == 0:
        return b""
    output = (ctypes.c_uint8 * len(input_data))()
    output_len = ctypes.c_size_t()
    input_ptr = ctypes.cast(ctypes.c_char_p(input_data), ctypes.POINTER(ctypes.c_uint8))
    res = lib_fn(
        ctx,
        input_ptr,
        len(input_data),
        output,
        len(input_data),
        ctypes.byref(output_len),
    )
    if res != 0:
        raise KelvinError("streaming update failed")
    return bytes(output[:output_len.value])


def _streaming_finalize(lib_fn, ctx) -> bytes:
    """Perform a streaming finalize and return the tag."""
    tag = (ctypes.c_uint8 * 64)()
    tag_len = ctypes.c_size_t()
    res = lib_fn(ctx, tag, 64, ctypes.byref(tag_len))
    if res != 0:
        raise KelvinError("streaming finalize failed")
    return bytes(tag[:tag_len.value])


def _streaming_decrypt_finalize(lib_fn, ctx, tag: bytes) -> None:
    """Perform a streaming decrypt finalize."""
    tag_ptr = ctypes.cast(ctypes.c_char_p(tag), ctypes.POINTER(ctypes.c_uint8)) if tag else None
    res = lib_fn(ctx, tag_ptr, len(tag))
    if res != 0:
        raise KelvinError("streaming decrypt finalize failed (tag mismatch)")


# ============================================================================
# Photon (V3) Streaming
# ============================================================================

class PhotonEncryptor:
    """V3 Photon streaming encryptor."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_photon_encryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_photon_encryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        seed_ptr = ctypes.cast(ctypes.c_char_p(seed), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_photon_encryptor_new(seed_ptr, len(seed), max_reseeds, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create PhotonEncryptor")
        self._ctx = ctx

    def update(self, plaintext: bytes) -> bytes:
        _lib.kelvin_photon_encryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_photon_encryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_photon_encryptor_update, self._ctx, plaintext)

    def finalize(self) -> bytes:
        _lib.kelvin_photon_encryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_photon_encryptor_finalize.restype = ctypes.c_int32
        return _streaming_finalize(_lib.kelvin_photon_encryptor_finalize, self._ctx)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_photon_encryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_photon_encryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# H Quantum Streaming
# ============================================================================

class QuantumEncryptor:
    """H Quantum streaming encryptor."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_quantum_encryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_quantum_encryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        seed_ptr = ctypes.cast(ctypes.c_char_p(seed), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_quantum_encryptor_new(seed_ptr, len(seed), max_reseeds, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create QuantumEncryptor")
        self._ctx = ctx

    def update(self, plaintext: bytes) -> bytes:
        _lib.kelvin_quantum_encryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_quantum_encryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_quantum_encryptor_update, self._ctx, plaintext)

    def finalize(self) -> bytes:
        _lib.kelvin_quantum_encryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_quantum_encryptor_finalize.restype = ctypes.c_int32
        return _streaming_finalize(_lib.kelvin_quantum_encryptor_finalize, self._ctx)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_quantum_encryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_quantum_encryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Chaos (V2) Streaming
# ============================================================================

class ChaosEncryptor:
    """V2 Chaos streaming encryptor."""

    def __init__(self, config_json: str, bytes_per_step: int = 65536):
        _lib.kelvin_chaos_encryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_chaos_encryptor_new.argtypes = [
            ctypes.c_char_p, ctypes.c_uint64, ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_chaos_encryptor_new(config_json.encode("utf-8"), bytes_per_step, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create ChaosEncryptor")
        self._ctx = ctx

    def update(self, plaintext: bytes) -> bytes:
        _lib.kelvin_chaos_encryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_chaos_encryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_chaos_encryptor_update, self._ctx, plaintext)

    def finalize(self) -> bytes:
        _lib.kelvin_chaos_encryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_chaos_encryptor_finalize.restype = ctypes.c_int32
        return _streaming_finalize(_lib.kelvin_chaos_encryptor_finalize, self._ctx)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_chaos_encryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_chaos_encryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Secure (V1) Streaming
# ============================================================================

class SecureEncryptor:
    """V1 Secure streaming encryptor (ChaCha20 + BLAKE3)."""

    def __init__(self, key: bytes, nonce: bytes):
        if len(key) != 32:
            raise ValueError("key must be 32 bytes")
        if len(nonce) != 12:
            raise ValueError("nonce must be 12 bytes")
        _lib.kelvin_secure_encryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_secure_encryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        key_ptr = ctypes.cast(ctypes.c_char_p(key), ctypes.POINTER(ctypes.c_uint8))
        nonce_ptr = ctypes.cast(ctypes.c_char_p(nonce), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_secure_encryptor_new(key_ptr, 32, nonce_ptr, 12, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create SecureEncryptor")
        self._ctx = ctx

    def update(self, plaintext: bytes) -> bytes:
        _lib.kelvin_secure_encryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_secure_encryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_secure_encryptor_update, self._ctx, plaintext)

    def finalize(self) -> bytes:
        _lib.kelvin_secure_encryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_secure_encryptor_finalize.restype = ctypes.c_int32
        return _streaming_finalize(_lib.kelvin_secure_encryptor_finalize, self._ctx)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_secure_encryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_secure_encryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Prism — OTP Key Generator for Homomorphic Encryption
# ============================================================================

class KelvinPrism:
    """Prism OTP key generator for homomorphic encryption."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_prism_new.restype = ctypes.c_void_p
        _lib.kelvin_prism_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        seed_ptr = ctypes.cast(ctypes.c_char_p(seed), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_prism_new(seed_ptr, len(seed), max_reseeds, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create Prism")
        self._ctx = ctx

    def generate_otp_key(self, length: int) -> bytes:
        _lib.kelvin_prism_generate_otp_key.restype = ctypes.c_int32
        _lib.kelvin_prism_generate_otp_key.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        output = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_prism_generate_otp_key(self._ctx, output, length)
        if res != 0:
            raise KelvinError("generate_otp_key failed")
        return bytes(output)

    def split_key(self, length: int) -> Tuple[bytes, bytes]:
        _lib.kelvin_prism_split_key.restype = ctypes.c_int32
        _lib.kelvin_prism_split_key.argtypes = [
            ctypes.c_void_p, ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.POINTER(ctypes.c_uint8),
        ]
        a = (ctypes.c_uint8 * length)()
        b = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_prism_split_key(self._ctx, length, a, b)
        if res != 0:
            raise KelvinError("split_key failed")
        return (bytes(a), bytes(b))

    def encrypt(self, data: bytearray) -> None:
        _lib.kelvin_prism_encrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        _lib.kelvin_prism_encrypt.restype = ctypes.c_int32
        buf = (ctypes.c_uint8 * len(data)).from_buffer(data)
        res = _lib.kelvin_prism_encrypt(self._ctx, buf, len(data))
        if res != 0:
            raise KelvinError("prism encrypt failed")

    def decrypt(self, data: bytearray) -> None:
        _lib.kelvin_prism_decrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        _lib.kelvin_prism_decrypt.restype = ctypes.c_int32
        buf = (ctypes.c_uint8 * len(data)).from_buffer(data)
        res = _lib.kelvin_prism_decrypt(self._ctx, buf, len(data))
        if res != 0:
            raise KelvinError("prism decrypt failed")

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_prism_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_prism_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Split — XOR Key-Splitter for Homomorphic Encryption
# ============================================================================

class KelvinSplit:
    """Split XOR key-splitter for homomorphic encryption."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_split_new.restype = ctypes.c_void_p
        _lib.kelvin_split_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        seed_ptr = ctypes.cast(ctypes.c_char_p(seed), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_split_new(seed_ptr, len(seed), max_reseeds, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create Split")
        self._ctx = ctx

    def generate_master_key(self, length: int) -> bytes:
        _lib.kelvin_split_generate_master_key.restype = ctypes.c_int32
        _lib.kelvin_split_generate_master_key.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        output = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_split_generate_master_key(self._ctx, output, length)
        if res != 0:
            raise KelvinError("generate_master_key failed")
        return bytes(output)

    def split_key(self, length: int) -> Tuple[bytes, bytes]:
        _lib.kelvin_split_key.restype = ctypes.c_int32
        _lib.kelvin_split_key.argtypes = [
            ctypes.c_void_p, ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.POINTER(ctypes.c_uint8),
        ]
        a = (ctypes.c_uint8 * length)()
        b = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_split_key(self._ctx, length, a, b)
        if res != 0:
            raise KelvinError("split_key failed")
        return (bytes(a), bytes(b))

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_split_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_split_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False


# ============================================================================
# Flare — Chaotic FHE Secret Key Generator
# ============================================================================

class KelvinFlare:
    """Flare chaotic FHE secret key generator."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_flare_new.restype = ctypes.c_void_p
        _lib.kelvin_flare_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        seed_ptr = ctypes.cast(ctypes.c_char_p(seed), ctypes.POINTER(ctypes.c_uint8))
        ctx = _lib.kelvin_flare_new(seed_ptr, len(seed), max_reseeds, ctypes.byref(error_out))
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create Flare")
        self._ctx = ctx

    def generate_secret_key(self, length: int) -> bytes:
        _lib.kelvin_flare_generate_secret_key.restype = ctypes.c_int32
        _lib.kelvin_flare_generate_secret_key.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t]
        output = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_flare_generate_secret_key(self._ctx, output, length)
        if res != 0:
            raise KelvinError("generate_secret_key failed")
        return bytes(output)

    # Scheme constants
    SCHEME_BFV = 0
    SCHEME_CKKS = 1
    SCHEME_TFHE = 2

    def generate_fhe_key(self, scheme: int, length: int) -> bytes:
        _lib.kelvin_flare_generate_fhe_key.restype = ctypes.c_int32
        _lib.kelvin_flare_generate_fhe_key.argtypes = [
            ctypes.c_void_p, ctypes.c_int32,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
        ]
        output = (ctypes.c_uint8 * length)()
        res = _lib.kelvin_flare_generate_fhe_key(self._ctx, scheme, output, length)
        if res != 0:
            raise KelvinError("generate_fhe_key failed")
        return bytes(output)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_flare_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_flare_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
        return False