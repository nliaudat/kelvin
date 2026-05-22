"""kelvin-pyo3 — Python bindings for the Kelvin Orbital Chaos KDF Cryptosystem.

Provides native Python wrappers for all Kelvin encryption modes:

- `generate_config()` — create a random orbital configuration
- `Kelvin` — V1 AEAD encrypt/decrypt (ChaCha20Poly1305)
- `KelvinPhoton` — V3 fast stream cipher (XOR-only)
- `KelvinQuantum` — H quantum-resistant stream cipher (XOR + reseeding)
- `KelvinPhotonAuthenticated` — V3 + BLAKE3-keyed MAC tag
- `KelvinQuantumAuthenticated` — H + BLAKE3-keyed MAC tag
- `KelvinStreaming` — V2 streaming mode (one step per chunk)

## Security

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

## Quick Start

```python
import kelvin_pyo3

# Generate a config
config = kelvin_pyo3.generate_config()

# V1 AEAD mode (16 extra bytes for AEAD tag)
k = kelvin_pyo3.Kelvin(config)
data = bytearray(b"Hello, world!" + b"\x00" * 16)
k.encrypt(data)
k.decrypt(data)
print(data[:13])  # b"Hello, world!"
```
"""

from kelvin_pyo3._core import (
    generate_config,
    Kelvin,
    KelvinPhoton,
    KelvinQuantum,
    KelvinPhotonAuthenticated,
    KelvinQuantumAuthenticated,
    KelvinStreaming,
)

__all__ = [
    "generate_config",
    "Kelvin",
    "KelvinPhoton",
    "KelvinQuantum",
    "KelvinPhotonAuthenticated",
    "KelvinQuantumAuthenticated",
    "KelvinStreaming",
]
