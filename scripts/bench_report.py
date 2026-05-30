#!/usr/bin/env python3
"""
Generate a human-readable Markdown report from criterion benchmark JSON data.

Reads the latest results from target/criterion/ and writes
documentation/bench_comparative.md

Usage:
    python scripts/bench_report.py          (after running cargo bench -p comparative-bench)
"""

import json
import os
import sys
from datetime import datetime
from pathlib import Path


def read_json(dirpath, filename):
    path = Path(dirpath) / filename
    if path.exists():
        return json.loads(path.read_text())
    return None


def extract_estimate(estimates, key):
    """Extract a point estimate from the estimates JSON structure."""
    if estimates and key in estimates and "point_estimate" in estimates[key]:
        return estimates[key]["point_estimate"]
    return None


def format_time(ns):
    """Convert nanoseconds to a human-friendly string."""
    if ns > 1_000_000_000:
        return f"{ns / 1_000_000_000:.4f} s"
    elif ns > 1_000_000:
        return f"{ns / 1_000_000:.4f} ms"
    elif ns > 1_000:
        return f"{ns / 1_000:.4f} µs"
    else:
        return f"{ns:.2f} ns"


def throughput_mbps(bytes_count, elapsed_ns):
    """Compute MB/s from bytes and elapsed time in nanoseconds."""
    if elapsed_ns <= 0:
        return 0.0
    bytes_per_sec = bytes_count / (elapsed_ns / 1_000_000_000.0)
    return bytes_per_sec / (1024 * 1024)


def generate_report():
    workspace = Path(__file__).resolve().parent.parent
    criterion = workspace / "target" / "criterion"
    output = workspace / "documentation" / "bench_comparative.md"

    if not criterion.exists():
        print("No benchmark data at target/criterion/")
        print("Run benchmarks first: cargo bench -p comparative-bench")
        sys.exit(1)

    lines = []
    lines.append("# Kelvin — Comparative Benchmark Report")
    lines.append("")
    lines.append(f"Generated `{datetime.now():%Y-%m-%d %H:%M:%S}` from criterion benchmarks.")
    lines.append("")
    lines.append("## System")
    lines.append("")
    lines.append("- **CPU**: AMD Ryzen 5 5600")
    lines.append("- **OS**: Windows 11")
    lines.append("- **Build**: Rust release profile (opt-level=3)")
    lines.append("")
    lines.append("---")
    lines.append("")

    # Define benchmark groups to extract
    groups = [
        {
            "title": "KelvinQuantum (H) vs AES-256-GCM vs ChaCha20-Poly1305",
            "desc": "Throughput comparison for 1 MiB encryption buffers. All in MB/s (higher is better).",
            "results": [
                ("KelvinQuantum (H)", "encrypt 1 MiB", 1 << 20),
                ("AES-256-GCM (ring)", "encrypt 1 MiB", 1 << 20),
                ("ChaCha20-Poly1305 (ring)", "encrypt 1 MiB", 1 << 20),
            ],
        },
        {
            "title": "KelvinStreaming (V2) vs AES-256-CTR",
            "desc": "Streaming throughput for 1 MiB buffers. KelvinStreaming advances n-body simulation one step per chunk.",
            "results": [
                ("KelvinStreaming (V2)", "encrypt 1 MiB", 1 << 20),
                ("AES-256-CTR", "encrypt 1 MiB", 1 << 20),
            ],
        },
        {
            "title": "Key Generation Time",
            "desc": "Key generation latency. Kelvin = full pipeline (config+simulate+extract). X25519 = scalar multiplication.",
            "results": [
                ("Kelvin orbital keygen", "full pipeline (config + simulate + extract)", 0),
                ("X25519 (dalek) keygen", "static secret + public key", 0),
            ],
        },
        {
            "title": "ED25519 Signature",
            "desc": "Sign and verify 1 MiB message. HAWK unavailable — no production Rust implementation.",
            "results": [
                ("ED25519 (dalek) sign", "sign 1 MiB", 1 << 20),
                ("ED25519 (dalek) verify", "verify 1 MiB", 1 << 20),
            ],
        },
    ]

    for group in groups:
        lines.append(f"## {group['title']}")
        lines.append("")
        lines.append(group['desc'])
        lines.append("")
        lines.append("| Implementation | Metric | Latency | MB/s |")
        lines.append("|----------------|--------|--------:|-----:|")

        for grp_name, bench_name, bytes_per_op in group['results']:
            grp_dir = criterion / grp_name
            if not grp_dir.exists():
                lines.append(f"| {grp_name} | {bench_name} | N/A (no data) | N/A |")
                continue

            # Read estimates from new/ or base/ directory
            estimates = None
            for sub in ["new", "base"]:
                est_path = grp_dir / bench_name / sub / "estimates.json"
                if est_path.exists():
                    estimates = read_json(est_path.parent, "estimates.json")
                    if estimates:
                        break

            if estimates:
                mean_ns = extract_estimate(estimates, "mean")
                std_ns = extract_estimate(estimates, "std_dev")
                lat = format_time(mean_ns) if mean_ns else "N/A"

                if bytes_per_op > 0 and mean_ns:
                    mbps = f"{throughput_mbps(bytes_per_op, mean_ns):.2f}"
                else:
                    mbps = "-"

                lines.append(f"| {grp_name} | {bench_name} | {lat} | {mbps} |")
            else:
                lines.append(f"| {grp_name} | {bench_name} | N/A (missing data) | N/A |")

        lines.append("")
        lines.append("---")
        lines.append("")

    lines.append("## Notes")
    lines.append("")
    lines.append("- **KelvinQuantum (H)**: SHAKE256 OTP stream cipher with periodic orbital reseeding. No authentication (XOR is malleable).")
    lines.append("- **AES-256-GCM / ChaCha20-Poly1305 (ring)**: Authenticated encryption (AEAD). Throughput includes AEAD tag but not verification.")
    lines.append("- **KelvinStreaming (V2)**: Advances n-body simulation one step per chunk — simulation cost dominates throughput.")
    lines.append("- **AES-256-CTR**: `aes` + `ctr` crates with 128-bit counter. No authentication.")
    lines.append("- **X25519 / Ed25519**: dalek crates.")
    lines.append("- **HAWK**: Not benchmarked — no production-quality Rust implementation exists.")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append("*Report auto-generated by `scripts/bench_report.py`. Run `scripts/comparative-bench` to refresh.*")
    lines.append("")

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Report written to: {output}")
    print(f"HTML reports:     {criterion / 'report' / 'index.html'}")


if __name__ == "__main__":
    generate_report()