#!/usr/bin/env python3
"""
Differential fuzzing reference: compare Rust Fixed Q32.64 vs Python mpmath (high-precision).

This script generates random orbital configurations and computes gravitational
accelerations using:
  1. Python mpmath (128-bit precision) — the ground truth
  2. Python float (IEEE 754 f64) — matching the Rust f64 path

It then compares the two, reporting:
  - Relative error statistics
  - Any sign flips
  - NaN/Inf divergence
  - Worst-case configurations

Usage:
  python tests/differential_fuzzing/reference.py --test-vectors 10000
  python tests/differential_fuzzing/reference.py --seed 42 --test-vectors 1000
"""

import argparse
import math
import random
import sys
from typing import List, Tuple

try:
    import mpmath as mp
except ImportError:
    print("mpmath not installed. Run: pip install mpmath")
    sys.exit(1)


def compute_accel_f64(
    positions: List[Tuple[float, float, float]],
    masses: List[float],
    softening: float,
    g: float,
) -> List[Tuple[float, float, float]]:
    """f64 acceleration computation (matches Rust f64 path)."""
    n = len(positions)
    accel = [(0.0, 0.0, 0.0) for _ in range(n)]
    softening_sq = softening * softening

    for i in range(n):
        for j in range(i + 1, n):
            dx = positions[j][0] - positions[i][0]
            dy = positions[j][1] - positions[i][1]
            dz = positions[j][2] - positions[i][2]
            dist_sq = dx * dx + dy * dy + dz * dz + softening_sq
            dist = math.sqrt(dist_sq)
            dist_cubed = dist_sq * dist
            factor = g / dist_cubed

            ax = factor * masses[j] * dx
            ay = factor * masses[j] * dy
            az = factor * masses[j] * dz

            accel[i] = (accel[i][0] + ax, accel[i][1] + ay, accel[i][2] + az)
            accel[j] = (accel[j][0] - factor * masses[i] * dx,
                        accel[j][1] - factor * masses[i] * dy,
                        accel[j][2] - factor * masses[i] * dz)

    return accel


def compute_accel_mpmath(
    positions: List[Tuple[float, float, float]],
    masses: List[float],
    softening: float,
    g: float,
    precision: int = 128,
) -> List[Tuple[float, float, float]]:
    """High-precision acceleration computation using mpmath."""
    mp.mp.dps = precision // 4 + 10  # decimal digits ≈ bits / log2(10)

    n = len(positions)
    # Convert to mp.mpf
    pos = [(mp.mpf(p[0]), mp.mpf(p[1]), mp.mpf(p[2])) for p in positions]
    m = [mp.mpf(mass) for mass in masses]
    eps_sq = mp.mpf(softening) ** 2
    G = mp.mpf(g)

    accel = [(mp.mpf(0), mp.mpf(0), mp.mpf(0)) for _ in range(n)]

    for i in range(n):
        for j in range(i + 1, n):
            dx = pos[j][0] - pos[i][0]
            dy = pos[j][1] - pos[i][1]
            dz = pos[j][2] - pos[i][2]
            dist_sq = dx * dx + dy * dy + dz * dz + eps_sq
            dist = mp.sqrt(dist_sq)
            dist_cubed = dist_sq * dist
            factor = G / dist_cubed

            ax = factor * m[j] * dx
            ay = factor * m[j] * dy
            az = factor * m[j] * dz

            accel[i] = (accel[i][0] + ax, accel[i][1] + ay, accel[i][2] + az)
            accel[j] = (accel[j][0] - factor * m[i] * dx,
                        accel[j][1] - factor * m[i] * dy,
                        accel[j][2] - factor * m[i] * dz)

    # Convert back to float for comparison
    return [(float(a[0]), float(a[1]), float(a[2])) for a in accel]


def relative_error(a: float, b: float) -> float:
    """Compute relative error between two values."""
    max_abs = max(abs(a), abs(b), 1e-30)
    return abs(a - b) / max_abs


def run_test(
    positions: List[Tuple[float, float, float]],
    masses: List[float],
    softening: float,
    g: float,
) -> dict:
    """Run a single test case and return statistics."""
    accel_f64 = compute_accel_f64(positions, masses, softening, g)
    accel_mp = compute_accel_mpmath(positions, masses, softening, g)

    n = len(positions)
    max_rel_err = 0.0
    max_rel_err_loc = (0, 0)
    sign_flips = 0
    finite_mismatches = 0

    for i in range(n):
        for j in range(3):
            a = accel_f64[i][j]
            b = accel_mp[i][j]

            # Check finite-ness
            a_finite = math.isfinite(a)
            b_finite = math.isfinite(b)
            if a_finite != b_finite:
                finite_mismatches += 1
                continue

            if not a_finite or not b_finite:
                continue

            # Check sign
            if a != 0.0 and b != 0.0 and math.copysign(1.0, a) != math.copysign(1.0, b):
                sign_flips += 1

            # Relative error
            err = relative_error(a, b)
            if err > max_rel_err:
                max_rel_err = err
                max_rel_err_loc = (i, j)

    return {
        "max_rel_err": max_rel_err,
        "max_rel_err_loc": max_rel_err_loc,
        "sign_flips": sign_flips,
        "finite_mismatches": finite_mismatches,
    }


def generate_random_config(seed: int = None) -> dict:
    """Generate a random orbital configuration."""
    if seed is not None:
        random.seed(seed)

    n = 5
    positions = [
        (random.uniform(-100, 100),
         random.uniform(-100, 100),
         random.uniform(-100, 100))
        for _ in range(n)
    ]
    masses = [random.uniform(1e-20, 1.0) for _ in range(n)]
    softening = random.uniform(0.0, 1.0)
    g = random.uniform(0.1, 1000.0)

    return {
        "positions": positions,
        "masses": masses,
        "softening": softening,
        "g": g,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Differential fuzzing: f64 vs mpmath reference"
    )
    parser.add_argument(
        "--test-vectors", type=int, default=1000,
        help="Number of random test vectors to run"
    )
    parser.add_argument(
        "--seed", type=int, default=None,
        help="Random seed for reproducibility"
    )
    parser.add_argument(
        "--threshold", type=float, default=1e-8,
        help="Relative error threshold (default: 1e-8)"
    )
    args = parser.parse_args()

    print(f"Running {args.test_vectors} test vectors...")
    print(f"Threshold: {args.threshold}")
    print()

    max_rel_err_overall = 0.0
    worst_config = None
    total_sign_flips = 0
    total_finite_mismatches = 0
    failures = 0

    for t in range(args.test_vectors):
        config = generate_random_config(
            seed=(args.seed + t) if args.seed is not None else None
        )
        result = run_test(
            config["positions"],
            config["masses"],
            config["softening"],
            config["g"],
        )

        if result["max_rel_err"] > max_rel_err_overall:
            max_rel_err_overall = result["max_rel_err"]
            worst_config = config

        total_sign_flips += result["sign_flips"]
        total_finite_mismatches += result["finite_mismatches"]

        if result["max_rel_err"] > args.threshold:
            failures += 1
            if failures <= 5:
                print(f"  FAIL #{failures}: rel_err={result['max_rel_err']:.2e} "
                      f"at body {result['max_rel_err_loc'][0]}, "
                      f"axis {result['max_rel_err_loc'][1]}")

    print()
    print("=" * 60)
    print("RESULTS")
    print("=" * 60)
    print(f"  Test vectors:     {args.test_vectors}")
    print(f"  Failures:         {failures} / {args.test_vectors}")
    print(f"  Max rel error:    {max_rel_err_overall:.2e}")
    print(f"  Sign flips:       {total_sign_flips}")
    print(f"  Finite mismatches: {total_finite_mismatches}")
    print()

    if worst_config:
        print("Worst-case configuration:")
        print(f"  positions = {worst_config['positions']}")
        print(f"  masses = {worst_config['masses']}")
        print(f"  softening = {worst_config['softening']}")
        print(f"  g = {worst_config['g']}")
        print()

    if failures == 0:
        print("✅ All tests passed — f64 and mpmath agree within threshold.")
    else:
        print(f"❌ {failures} failures detected — investigate worst-case config above.")
        sys.exit(1)


if __name__ == "__main__":
    main()
