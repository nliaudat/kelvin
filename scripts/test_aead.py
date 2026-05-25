#!/usr/bin/env python3
"""Quick test for AEAD encrypt/decrypt round-trip."""
import subprocess
import hashlib
import json
import sys
from pathlib import Path

workspace = Path(__file__).resolve().parent.parent
kelvin_exe = workspace / "target" / "release" / "kelvin.exe"
temp_dir = Path("D:/temp/kelvin_temp")
temp_dir.mkdir(parents=True, exist_ok=True)

key_file = temp_dir / "key.json"
test_file = temp_dir / "test.bin"
enc_file = temp_dir / "test.enc"
dec_file = temp_dir / "test_dec.bin"

# Generate key
r = subprocess.run([str(kelvin_exe), "keygen", "--level", "paranoid", "--output", str(key_file)],
                   capture_output=True, text=True)
print(f"Keygen: rc={r.returncode}")
if r.returncode != 0:
    print(r.stderr)
    sys.exit(1)

# Override steps to 110000
with open(key_file) as f:
    config = json.load(f)
config["total_steps"] = 110000
config["reseed_interval"] = 11000
with open(key_file, "w") as f:
    json.dump(config, f)
print(f"Config steps: {config['total_steps']}")

# Create 2 MB test file
with open(test_file, "wb") as f:
    f.write(b"\x00\x01" * (1024 * 1024))
print(f"Test file: {test_file.stat().st_size} bytes")

# Encrypt
r = subprocess.run([str(kelvin_exe), "encrypt", "--mode", "secure",
                    "--config", str(key_file), "--input", str(test_file),
                    "--output", str(enc_file)],
                   capture_output=True, text=True)
print(f"Encrypt: rc={r.returncode}")
if r.returncode != 0:
    print("STDERR:", r.stderr)
    sys.exit(1)

# Decrypt
r = subprocess.run([str(kelvin_exe), "decrypt", "--mode", "secure",
                    "--config", str(key_file), "--input", str(enc_file),
                    "--output", str(dec_file)],
                   capture_output=True, text=True)
print(f"Decrypt: rc={r.returncode}")
if r.returncode != 0:
    print("STDERR:", r.stderr)
    sys.exit(1)

# Verify
h1 = hashlib.sha256(test_file.read_bytes()).hexdigest()
h2 = hashlib.sha256(dec_file.read_bytes()).hexdigest()
match = h1 == h2
print(f"SHA256 match: {match}")
print(f"  Input:  {h1}")
print(f"  Output: {h2}")

if match:
    print("\n*** AEAD FIX VERIFIED ***")
else:
    print("\n*** AEAD FIX FAILED ***")
    sys.exit(1)
