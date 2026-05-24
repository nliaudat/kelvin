#!/usr/bin/env python3
"""
Kelvin -- Encryption Benchmark (Cross-Platform)

Benchmarks all modes (chaos, photon, quantum) on a file of configurable size
with alternating 0x00/0x01 pattern. Outputs results to documentation/bench_*.md

Usage:
    python scripts/bench.py              (default: 1 GB, paranoid level)
    python scripts/bench.py 100          (100 GB)
    python scripts/bench.py 10           (10 GB)
    python scripts/bench.py 1 --level maximum
"""

import argparse
import hashlib
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path

# Test pattern: alternating 0x00/0x01 bytes
PATTERN = b'\x00\x01'


def parse_args():
    parser = argparse.ArgumentParser(description="Kelvin Encryption Benchmark")
    parser.add_argument("size_gb", nargs="?", type=int, default=1,
                        help="File size in GB (default: 1)")
    parser.add_argument("--level", choices=["standard", "paranoid", "maximum"],
                        default="paranoid",
                        help="Keygen security level (default: paranoid)")
    parser.add_argument("--bytes-per-step", type=int, default=1048576,
                        help="Bytes per step/chunk for chaos/photon/quantum (default: 1048576 = 1 MiB)")
    parser.add_argument("--continue-on-error", action="store_true",
                        help="Continue to next mode if a mode fails (instead of aborting)")
    return parser.parse_args()


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    """Run a command and return the result."""
    print(f"  Running: {' '.join(cmd)}")
    return subprocess.run(cmd, **kwargs)


def main():
    args = parse_args()
    size_gb = args.size_gb
    level = args.level
    size_bytes = size_gb * 1024**3

    # Paths
    workspace = Path(__file__).resolve().parent.parent
    kelvin_exe = workspace / "target" / "release" / "kelvin.exe"
    temp_dir = Path("D:/temp/kelvin_temp")
    key_file = temp_dir / "key.json"
    input_file = temp_dir / f"test_{size_gb}gb.bin"
    enc_file = temp_dir / f"test_{size_gb}gb.enc"
    dec_file = temp_dir / f"test_{size_gb}gb_dec.bin"
    report = workspace / "documentation" / f"bench_{size_gb}gb.md"

    os.chdir(workspace)

    print(f"\033[96m{'=' * 40}\033[0m")
    print(f"\033[96m  Kelvin {size_gb} GB Encryption Benchmark\033[0m")
    print(f"\033[96m{'=' * 40}\033[0m")
    print()

    # -----------------------------------------------------------------------
    # 1. Setup
    # -----------------------------------------------------------------------
    print("\033[96m[1/7] Creating temp directory...\033[0m")
    temp_dir.mkdir(parents=True, exist_ok=True)

    print("\033[96m[2/7] Building release CLI...\033[0m")
    result = run(["cargo", "build", "--release", "-p", "kelvin-cli"])
    if result.returncode != 0:
        print("\033[91mBuild failed\033[0m")
        sys.exit(1)
    print("\033[92m  Done\033[0m")

    print(f"\033[96m[3/7] Generating orbital config ({level} level)...\033[0m")
    result = run([str(kelvin_exe), "keygen", "--level", level, "--output", str(key_file)],
                 capture_output=True, text=True)
    if result.returncode != 0:
        print("\033[91mKeygen failed\033[0m")
        print(result.stdout)
        print(result.stderr)
        sys.exit(1)
    # Capture keygen summary info
    keygen_info = result.stdout.strip() if result.stdout else ""
    if not keygen_info:
        keygen_info = result.stderr.strip() if result.stderr else ""
    print("\033[92m  Done\033[0m")
    if keygen_info:
        for line in keygen_info.splitlines():
            print(f"    {line}")
    print()


    print(f"\033[96m[4/7] Generating {size_gb} GB pattern file...\033[0m")
    print(f"\033[93m  Writing pattern: {PATTERN!r}\033[0m")
    chunk_size = 16 * 1024**2  # 16 MB chunks
    chunk = PATTERN * (chunk_size // len(PATTERN))
    with open(input_file, 'wb', buffering=1024*1024) as f:
        remaining = size_bytes
        while remaining > 0:
            n = min(chunk_size, remaining)
            f.write(chunk[:n])
            remaining -= n
            sys.stderr.write('.')
            sys.stderr.flush()
    sys.stderr.write('\n')
    print("\033[92m  Done\033[0m")

    print("\033[96m[5/7] Computing input file hash...\033[0m")
    sha256 = hashlib.sha256()
    with open(input_file, 'rb') as f:
        while True:
            data = f.read(64 * 1024**2)
            if not data:
                break
            sha256.update(data)
    input_hash = sha256.hexdigest()
    print(f"\033[92m  SHA256: {input_hash}\033[0m")

    # -----------------------------------------------------------------------
    # 2. Benchmark
    # -----------------------------------------------------------------------
    print()
    print(f"\033[96m{'=' * 40}\033[0m")
    print(f"\033[96m  Running Benchmarks\033[0m")
    print(f"\033[96m{'=' * 40}\033[0m")
    print()

    print("\033[96m[6/7] Running encrypt/decrypt/verify for each mode...\033[0m")

    # Initialize report
    with open(report, 'w') as f:
        f.write(f"# Kelvin {size_gb} GB Encryption Benchmark\n\n")
        f.write(f"**Date:** {time.strftime('%d.%m.%Y %H:%M:%S')}\n")
        f.write("**Platform:** Cross-Platform (Python)\n")
        f.write(f"**Test file:** {size_gb} GB pattern {PATTERN!r}\n")
        f.write("**Integration:** Verlet (default)\n")
        f.write(f"**Key level:** {level}\n")
        if keygen_info:
            f.write(f"**Keygen output:**\n```\n{keygen_info}\n```\n")
        f.write("\n| Mode | Operation | Time (s) | Throughput (GB/s) | Throughput (MB/s) | Verify |\n")
        f.write("|---|-----------|----------|-------------------|--------------------|--------|\n")


    modes = ["chaos", "photon", "quantum"]
    results = {}  # mode -> {encrypt: status, decrypt: status, verify: status}

    for mode in modes:
        print()
        print(f"\033[96m--- Mode: {mode} ---\033[0m")

        # Encrypt
        print("\033[93m  Encrypting...\033[0m")
        t0 = time.time()
        cmd = [str(kelvin_exe), "encrypt", "--mode", mode,
               "--config", str(key_file), "--input", str(input_file),
               "--output", str(enc_file)]
        # Only pass --bytes-per-step for chaos mode; photon/quantum use internal defaults
        if mode == "chaos":
            cmd.extend(["--bytes-per-step", str(args.bytes_per_step)])
        result = run(cmd)
        t1 = time.time()
        if result.returncode != 0:
            print(f"\033[91m  Encryption failed (rc={result.returncode})\033[0m")
            enc_time = "ERROR"
            enc_gbps = 0
            enc_mbps = 0
            if not args.continue_on_error:
                print("\033[91m  Aborting (use --continue-on-error to skip failed modes)\033[0m")
                sys.exit(1)
        else:
            enc_time = f"{t1 - t0:.3f}"
            enc_gbps = f"{size_gb / (t1 - t0):.3f}"
            enc_mbps = f"{(size_gb * 1024) / (t1 - t0):.1f}"
            print(f"\033[92m  Encrypt time: {enc_time} s\033[0m")

        # Decrypt
        print("\033[93m  Decrypting...\033[0m")
        t0 = time.time()
        cmd = [str(kelvin_exe), "decrypt", "--mode", mode,
               "--config", str(key_file), "--input", str(enc_file),
               "--output", str(dec_file)]
        # Only pass --bytes-per-step for chaos mode; photon/quantum use internal defaults
        if mode == "chaos":
            cmd.extend(["--bytes-per-step", str(args.bytes_per_step)])
        result = run(cmd)
        t1 = time.time()
        if result.returncode != 0:
            print(f"\033[91m  Decryption failed (rc={result.returncode})\033[0m")
            dec_time = "ERROR"
            dec_gbps = 0
            dec_mbps = 0
            if not args.continue_on_error:
                print("\033[91m  Aborting (use --continue-on-error to skip failed modes)\033[0m")
                sys.exit(1)
        else:
            dec_time = f"{t1 - t0:.3f}"
            dec_gbps = f"{size_gb / (t1 - t0):.3f}"
            dec_mbps = f"{(size_gb * 1024) / (t1 - t0):.1f}"
            print(f"\033[92m  Decrypt time: {dec_time} s\033[0m")

        # Verify (skip if encryption or decryption failed)
        verify = "FAIL"
        if enc_time != "ERROR" and dec_time != "ERROR" and dec_file.exists():
            sha256_dec = hashlib.sha256()
            with open(dec_file, 'rb') as f:
                while True:
                    data = f.read(64 * 1024**2)
                    if not data:
                        break
                    sha256_dec.update(data)
            dec_hash = sha256_dec.hexdigest()
            verify = "PASS" if dec_hash == input_hash else "FAIL"
            if verify == "PASS":
                print(f"\033[92m  SHA256 match: PASS\033[0m")
            else:
                print(f"\033[91m  SHA256 MISMATCH!\033[0m")

        # Verify encrypted file does NOT contain the plaintext pattern
        print("\033[93m  Checking for pattern leak in ciphertext...\033[0m")
        pattern_check = "N/A"
        if enc_time != "ERROR" and enc_file.exists():
            # Sample the encrypted file to check for the pattern
            with open(enc_file, 'rb') as f:
                sample = f.read(8192)  # Read first 8 KB
            if PATTERN in sample:
                pattern_check = "FAIL"
                print(f"\033[91m  WARNING: Plaintext pattern found in encrypted file!\033[0m")
            else:
                pattern_check = "PASS"
                print(f"\033[92m  Pattern not found in ciphertext: PASS\033[0m")

        # Append to report
        with open(report, 'a') as f:
            f.write(f"| {mode} | encrypt | {enc_time} | {enc_gbps} | {enc_mbps} | {verify} |\n")
            f.write(f"| {mode} | decrypt | {dec_time} | {dec_gbps} | {dec_mbps} | {verify} |\n")

        # Track results for summary
        results[mode] = {
            "encrypt": "PASS" if enc_time != "ERROR" else "FAIL",
            "decrypt": "PASS" if dec_time != "ERROR" else "FAIL",
            "verify": verify,
        }

        # Clean up intermediate files
        if enc_file.exists():
            enc_file.unlink()
        if dec_file.exists():
            dec_file.unlink()

    # Close report
    with open(report, 'a') as f:
        f.write("|---|-----------|----------|-------------------|--------------------|--------|\n")
        f.write(f"\n*Benchmark completed at {time.strftime('%d.%m.%Y %H:%M:%S')}*\n")
        f.write("\n## Summary\n\n")
        f.write("| Mode | Encrypt | Decrypt | Verify |\n")
        f.write("|------|---------|---------|--------|\n")
        for mode in modes:
            r = results.get(mode, {"encrypt": "SKIP", "decrypt": "SKIP", "verify": "SKIP"})
            f.write(f"| {mode} | {r['encrypt']} | {r['decrypt']} | {r['verify']} |\n")

    # -----------------------------------------------------------------------
    # 3. Cleanup
    # -----------------------------------------------------------------------
    print()
    print("\033[96m[7/7] Cleaning up...\033[0m")
    if input_file.exists():
        input_file.unlink()
    if key_file.exists():
        key_file.unlink()
    shutil.rmtree(temp_dir, ignore_errors=True)

    print()
    print(f"\033[92m{'=' * 40}\033[0m")
    print(f"\033[92m  Benchmark Complete\033[0m")
    print(f"\033[92m{'=' * 40}\033[0m")
    print()
    print(f"Report written to: {report}")
    with open(report) as f:
        print(f.read())


if __name__ == "__main__":
    main()
