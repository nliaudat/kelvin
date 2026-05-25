#!/usr/bin/env python3
"""Compare Photon/Quantum throughput at different chunk sizes."""
import subprocess
import hashlib
import json
import time
import sys
from pathlib import Path

workspace = Path(__file__).resolve().parent.parent
kelvin_exe = workspace / "target" / "release" / "kelvin.exe"
temp_dir = Path("D:/temp/kelvin_temp")
temp_dir.mkdir(parents=True, exist_ok=True)

key_file = temp_dir / "key.json"
test_file = temp_dir / "test_256mb.bin"
enc_file = temp_dir / "test_256mb.enc"
dec_file = temp_dir / "test_256mb_dec.bin"

# Generate key (fast mode: 110,000 steps instead of 10M)
print("Generating key (fast mode)...")
r = subprocess.run([str(kelvin_exe), "keygen", "--level", "paranoid", "--fast", "--output", str(key_file)],
                   capture_output=True, text=True)
if r.returncode != 0:
    print("Keygen failed:", r.stderr)
    sys.exit(1)

# Create 256 MB test file
print("Creating 256 MB test file...")
with open(test_file, "wb") as f:
    chunk = b"\x00\x01" * (8 * 1024 * 1024)  # 16 MB
    for _ in range(16):
        f.write(chunk)

# Compute hash
h_input = hashlib.sha256(test_file.read_bytes()).hexdigest()
print(f"Input hash: {h_input}")

chunk_sizes = [1048576, 67108864, 268435456]  # 1 MB, 64 MB, 256 MB
labels = ["1 MB", "64 MB", "256 MB"]

for mode in ["photon", "quantum"]:
    print(f"\n{'='*50}")
    print(f"  Mode: {mode}")
    print(f"{'='*50}")
    
    for chunk_size, label in zip(chunk_sizes, labels):
        print(f"\n  --- Chunk size: {label} ---")
        
        # Encrypt
        t0 = time.time()
        r = subprocess.run([str(kelvin_exe), "encrypt", "--mode", mode,
                            "--config", str(key_file), "--input", str(test_file),
                            "--output", str(enc_file),
                            "--bytes-per-step", str(chunk_size)],
                           capture_output=True, text=True)
        t1 = time.time()
        if r.returncode != 0:
            print(f"  Encrypt FAILED: {r.stderr[:200]}")
            continue
        enc_time = t1 - t0
        enc_mbps = (256 / enc_time) * 1024 / 1024  # MB/s
        print(f"  Encrypt: {enc_time:.3f}s = {enc_mbps:.1f} MB/s")
        
        # Decrypt
        t0 = time.time()
        r = subprocess.run([str(kelvin_exe), "decrypt", "--mode", mode,
                            "--config", str(key_file), "--input", str(enc_file),
                            "--output", str(dec_file),
                            "--bytes-per-step", str(chunk_size)],
                           capture_output=True, text=True)
        t1 = time.time()
        if r.returncode != 0:
            print(f"  Decrypt FAILED: {r.stderr[:200]}")
            continue
        dec_time = t1 - t0
        dec_mbps = (256 / dec_time) * 1024 / 1024
        print(f"  Decrypt: {dec_time:.3f}s = {dec_mbps:.1f} MB/s")
        
        # Verify
        h_dec = hashlib.sha256(dec_file.read_bytes()).hexdigest()
        verify = "PASS" if h_dec == h_input else "FAIL"
        print(f"  Verify: {verify}")
        
        # Cleanup
        if enc_file.exists():
            enc_file.unlink()
        if dec_file.exists():
            dec_file.unlink()

print("\nDone!")
