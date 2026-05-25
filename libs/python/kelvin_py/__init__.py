"""
Kelvin Cryptosystem — Python bindings via C FFI.

Provides both the V1 AEAD API and the streaming encrypt/decrypt API
for all 4 modes (Photon, Quantum, Chaos, Secure).
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

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


# ============================================================================
# Streaming API — Generic helpers
# ============================================================================

def _streaming_update(lib_fn, ctx, input_data: bytes) -> bytes:
    """Perform a streaming update and return the output."""
    if len(input_data) == 0:
        return b""
    output = (ctypes.c_uint8 * len(input_data))()
    output_len = ctypes.c_size_t()
    res = lib_fn(
        ctx,
        ctypes.cast(input_data, ctypes.POINTER(ctypes.c_uint8)),
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
    tag_ptr = ctypes.cast(tag, ctypes.POINTER(ctypes.c_uint8)) if tag else None
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
        ctx = _lib.kelvin_photon_encryptor_new(
            ctypes.cast(seed, ctypes.POINTER(ctypes.c_uint8)),
            len(seed), max_reseeds, ctypes.byref(error_out),
        )
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


class PhotonDecryptor:
    """V3 Photon streaming decryptor."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_photon_decryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_photon_decryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_photon_decryptor_new(
            ctypes.cast(seed, ctypes.POINTER(ctypes.c_uint8)),
            len(seed), max_reseeds, ctypes.byref(error_out),
        )
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create PhotonDecryptor")
        self._ctx = ctx

    def update(self, ciphertext: bytes) -> bytes:
        _lib.kelvin_photon_decryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_photon_decryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_photon_decryptor_update, self._ctx, ciphertext)

    def finalize(self, tag: bytes = b"") -> None:
        _lib.kelvin_photon_decryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
        ]
        _lib.kelvin_photon_decryptor_finalize.restype = ctypes.c_int32
        _streaming_decrypt_finalize(_lib.kelvin_photon_decryptor_finalize, self._ctx, tag)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_photon_decryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_photon_decryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


# ============================================================================
# Quantum (H) Streaming
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
        ctx = _lib.kelvin_quantum_encryptor_new(
            ctypes.cast(seed, ctypes.POINTER(ctypes.c_uint8)),
            len(seed), max_reseeds, ctypes.byref(error_out),
        )
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


class QuantumDecryptor:
    """H Quantum streaming decryptor."""

    def __init__(self, seed: bytes, max_reseeds: int = 1000):
        _lib.kelvin_quantum_decryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_quantum_decryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.c_uint64,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_quantum_decryptor_new(
            ctypes.cast(seed, ctypes.POINTER(ctypes.c_uint8)),
            len(seed), max_reseeds, ctypes.byref(error_out),
        )
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create QuantumDecryptor")
        self._ctx = ctx

    def update(self, ciphertext: bytes) -> bytes:
        _lib.kelvin_quantum_decryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_quantum_decryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_quantum_decryptor_update, self._ctx, ciphertext)

    def finalize(self, tag: bytes = b"") -> None:
        _lib.kelvin_quantum_decryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
        ]
        _lib.kelvin_quantum_decryptor_finalize.restype = ctypes.c_int32
        _streaming_decrypt_finalize(_lib.kelvin_quantum_decryptor_finalize, self._ctx, tag)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_quantum_decryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_quantum_decryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


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
        ctx = _lib.kelvin_chaos_encryptor_new(
            config_json.encode("utf-8"), bytes_per_step, ctypes.byref(error_out),
        )
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


class ChaosDecryptor:
    """V2 Chaos streaming decryptor."""

    def __init__(self, config_json: str, bytes_per_step: int = 65536):
        _lib.kelvin_chaos_decryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_chaos_decryptor_new.argtypes = [
            ctypes.c_char_p, ctypes.c_uint64, ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_chaos_decryptor_new(
            config_json.encode("utf-8"), bytes_per_step, ctypes.byref(error_out),
        )
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create ChaosDecryptor")
        self._ctx = ctx

    def update(self, ciphertext: bytes) -> bytes:
        _lib.kelvin_chaos_decryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_chaos_decryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_chaos_decryptor_update, self._ctx, ciphertext)

    def finalize(self, tag: bytes = b"") -> None:
        _lib.kelvin_chaos_decryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
        ]
        _lib.kelvin_chaos_decryptor_finalize.restype = ctypes.c_int32
        _streaming_decrypt_finalize(_lib.kelvin_chaos_decryptor_finalize, self._ctx, tag)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_chaos_decryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_chaos_decryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()


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
        ctx = _lib.kelvin_secure_encryptor_new(
            ctypes.cast(key, ctypes.POINTER(ctypes.c_uint8)), 32,
            ctypes.cast(nonce, ctypes.POINTER(ctypes.c_uint8)), 12,
            ctypes.byref(error_out),
        )
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


class SecureDecryptor:
    """V1 Secure streaming decryptor."""

    def __init__(self, key: bytes, nonce: bytes):
        if len(key) != 32:
            raise ValueError("key must be 32 bytes")
        if len(nonce) != 12:
            raise ValueError("nonce must be 12 bytes")
        _lib.kelvin_secure_decryptor_new.restype = ctypes.c_void_p
        _lib.kelvin_secure_decryptor_new.argtypes = [
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_char_p),
        ]
        error_out = ctypes.c_char_p()
        ctx = _lib.kelvin_secure_decryptor_new(
            ctypes.cast(key, ctypes.POINTER(ctypes.c_uint8)), 32,
            ctypes.cast(nonce, ctypes.POINTER(ctypes.c_uint8)), 12,
            ctypes.byref(error_out),
        )
        if not ctx:
            raise KelvinError(error_out.value.decode("utf-8") if error_out.value else "failed to create SecureDecryptor")
        self._ctx = ctx

    def update(self, ciphertext: bytes) -> bytes:
        _lib.kelvin_secure_decryptor_update.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
            ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t, ctypes.POINTER(ctypes.c_size_t),
        ]
        _lib.kelvin_secure_decryptor_update.restype = ctypes.c_int32
        return _streaming_update(_lib.kelvin_secure_decryptor_update, self._ctx, ciphertext)

    def finalize(self, tag: bytes) -> None:
        _lib.kelvin_secure_decryptor_finalize.argtypes = [
            ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint8), ctypes.c_size_t,
        ]
        _lib.kelvin_secure_decryptor_finalize.restype = ctypes.c_int32
        _streaming_decrypt_finalize(_lib.kelvin_secure_decryptor_finalize, self._ctx, tag)

    def close(self) -> None:
        if self._ctx:
            _lib.kelvin_secure_decryptor_free.argtypes = [ctypes.c_void_p]
            _lib.kelvin_secure_decryptor_free(self._ctx)
            self._ctx = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()
