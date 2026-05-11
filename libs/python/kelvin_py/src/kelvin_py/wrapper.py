import ctypes
import os
import json
from typing import Optional

class KelvinError(Exception):
    pass

class Kelvin:
    """
    Python wrapper for the Kelvin cryptosystem.
    Requires the kelvin_ffi shared library (libkelvin_ffi.so, kelvin_ffi.dll, or libkelvin_ffi.dylib).
    """
    
    def __init__(self, config_json: str, lib_path: Optional[str] = None):
        if lib_path is None:
            # Default to looking in current directory or system paths
            lib_name = "kelvin_ffi.dll" if os.name == "nt" else "libkelvin_ffi.so"
            lib_path = os.path.join(os.path.dirname(__file__), lib_name)
        
        try:
            self._lib = ctypes.CDLL(lib_path)
        except OSError as e:
            raise KelvinError(f"Could not load Kelvin FFI library at {lib_path}: {e}")

        # Define function signatures
        self._lib.kelvin_new.restype = ctypes.c_void_p
        self._lib.kelvin_new.argtypes = [ctypes.c_char_p, ctypes.POINTER(ctypes.c_char_p)]
        
        self._lib.kelvin_encrypt.restype = ctypes.c_int32
        self._lib.kelvin_encrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t]
        
        self._lib.kelvin_decrypt.restype = ctypes.c_int32
        self._lib.kelvin_decrypt.argtypes = [ctypes.c_void_p, ctypes.POINTER(ctypes.c_ubyte), ctypes.c_size_t]
        
        self._lib.kelvin_remaining_bytes.restype = ctypes.c_uint64
        self._lib.kelvin_remaining_bytes.argtypes = [ctypes.c_void_p]
        
        self._lib.kelvin_free.restype = None
        self._lib.kelvin_free.argtypes = [ctypes.c_void_p]

        # Initialize context
        error_ptr = ctypes.c_char_p()
        self._ctx = self._lib.kelvin_new(config_json.encode('utf-8'), ctypes.byref(error_ptr))
        
        if not self._ctx:
            error_msg = error_ptr.value.decode('utf-8') if error_ptr.value else "Unknown error"
            raise KelvinError(f"Failed to initialize Kelvin: {error_msg}")

    def encrypt(self, data: bytearray):
        """Encrypt data in-place."""
        size = len(data)
        data_ptr = (ctypes.c_ubyte * size).from_buffer(data)
        res = self._lib.kelvin_encrypt(self._ctx, data_ptr, size)
        if res != 0:
            raise KelvinError("Encryption failed")

    def decrypt(self, data: bytearray):
        """Decrypt data in-place."""
        # Encryption and decryption are identical in Kelvin (ChaCha20 XOR)
        self.encrypt(data)

    @property
    def remaining_bytes(self) -> int:
        """Return remaining safe bytes before simulation exhaustion."""
        return self._lib.kelvin_remaining_bytes(self._ctx)

    def __del__(self):
        if hasattr(self, '_ctx') and self._ctx:
            self._lib.kelvin_free(self._ctx)
            self._ctx = None

def generate_standard_config():
    """Example helper to generate a standard configuration JSON (place holder)."""
    # In a real implementation, this would call the keygen logic or a server
    return json.dumps({
        "bodies": [], # ... would contain real bodies
        "total_steps": 1000000,
        "reseed_interval": 100000,
        "dt": {"raw": 18014398509481984},
        "softening": {"raw": 72057594037927936},
        "g": {"raw": 72813134371493658624}
    })
