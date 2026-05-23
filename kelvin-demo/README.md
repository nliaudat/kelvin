========================================
  KELVIN CRYPTOSYSTEM - DEMO KIT
  Orbital Chaos KDF - Post-Quantum Cryptography
========================================

WHAT'S INCLUDED
---------------
  kelvin.exe              - CLI: keygen, encrypt, decrypt, identify
  kelvin-test-client.exe  - Integration test client (V1 + V2 streaming self-tests)
  kelvin-test-server.exe  - Integration test server
  keygen_identify.exe     - Key generation + identity demo
  simple_encrypt.exe      - Simple encrypt/decrypt demo (V1 ChaCha20)
  simple_streaming.exe    - V2 streaming encrypt/decrypt demo (SHAKE256 XOR)
  orbital_visualizer.html - 3D orbital simulation with KDF pipeline display
  sample_key.json         - Sample orbital configuration
   (see scripts/test-all.bat for the full test suite)

QUICK START
-----------
  1. Generate a key:
     .\kelvin.exe keygen --output mykey.json

  2. Identify public keys:
     .\kelvin.exe identify --config mykey.json

  3. Encrypt a file:
     .\kelvin.exe encrypt --config mykey.json --input secret.txt --output secret.enc

  4. Decrypt a file:
     .\kelvin.exe decrypt --config mykey.json --input secret.enc --output secret.txt

  5. Open orbital_visualizer.html in a browser for 3D visualization.
     - Drag to rotate, scroll to zoom, Space/P to pause, R to reset
     - Load a key.json file to visualize custom orbital dynamics
     - Hover over KDF pipeline stages for detailed information

  6. Run all tests (from the repo root):
     ..\scripts\test-all.bat

DEMO BINARIES
-------------
  keygen_identify.exe:
     Generates a random orbital config and derives hybrid PQ key pairs.
     Usage: .\keygen_identify.exe

  simple_encrypt.exe:
     Encrypts/decrypts a hardcoded message using V1 (ChaCha20Poly1305 AEAD).
     Usage: .\simple_encrypt.exe

  simple_streaming.exe:
     Demonstrates V2 streaming encrypt/decrypt using SHAKE256 XOR.
     Each call advances the orbital simulation by one Verlet step.
     Usage: .\simple_streaming.exe

V2 STREAMING MODE
-----------------
  The V2 streaming mode (KelvinStreaming) provides:
  - Unlimited keystream: keep simulating as long as needed
  - Instant setup: no upfront simulation required
  - Deterministic: same config + same bytes = same keystream
  - Fixed-size chunking: data is processed in bytes_per_step chunks
  - Benchmarking: measure simulation speed for ETA estimation

  See documentation/usage.md Section 6 for full API details.

NOTES
-----
  - Use --fast with identify for quick key display (caps at 10K steps).
    WARNING: --fast produces a non-deterministic public key that will NOT
    match the key used for normal encryption/decryption. Use without --fast
    for consistent identification.
  - Without --fast, identify runs the full simulation (may be slow).
  - All binaries are 64-bit Windows executables.
  - Requires no external dependencies.

BUILD DATE: 2026-05-18 22:20
