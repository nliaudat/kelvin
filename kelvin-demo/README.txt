========================================
  KELVIN CRYPTOSYSTEM - DEMO KIT
  Orbital Chaos KDF - Post-Quantum Cryptography
========================================

WHAT'S INCLUDED
---------------
  kelvin.exe              - CLI: keygen, encrypt, decrypt, identify
  kelvin-test-client.exe  - Integration test client
  kelvin-test-server.exe  - Integration test server
  keygen_identify.exe     - Key generation + identity demo
  simple_encrypt.exe      - Simple encrypt/decrypt demo
  simple_streaming.exe    - Streaming encrypt/decrypt demo
  orbital_visualizer.html - 3D orbital simulation (open in browser)
  sample_key.json         - Sample orbital configuration
  test-all.ps1            - Windows test suite
  test-all.sh             - Linux test suite

QUICK START
-----------
  1. Generate a key:
     .\kelvin.exe keygen --output mykey.json

  2. Identify public keys:
     .\kelvin.exe identify --config mykey.json --fast

  3. Encrypt a file:
     .\kelvin.exe encrypt --config mykey.json --input secret.txt --output secret.enc

  4. Decrypt a file:
     .\kelvin.exe decrypt --config mykey.json --input secret.enc --output secret.txt

  5. Open orbital_visualizer.html in a browser for 3D visualization.

  6. Run all tests:
     .\test-all.ps1

DEMO BINARIES
-------------
  keygen_identify.exe:
     Generates a random orbital config and derives hybrid PQ key pairs.
     Usage: .\keygen_identify.exe

  simple_encrypt.exe:
     Encrypts/decrypts a hardcoded message using V1 (ChaCha20).
     Usage: .\simple_encrypt.exe

  simple_streaming.exe:
     Demonstrates V2 streaming encrypt/decrypt.
     Usage: .\simple_streaming.exe

NOTES
-----
  - Use --fast with identify for quick key display (caps at 10K steps).
  - Without --fast, identify runs the full simulation (may be slow).
  - All binaries are 64-bit Windows executables.
  - Requires no external dependencies.

BUILD DATE: 2026-05-18 22:20
