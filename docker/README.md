# Docker — Kani Formal Verification

This directory provides a **reproducible Docker environment** for running Kani
model-checking proof harnesses in the Kelvin cryptosystem.
The active proof suite is 37 harnesses: 8 in `kelvin-core/src/fixed_math.rs` plus 29
in `proofs/kani/` included by `kelvin-core/src/kani_harnesses.rs`. Other files in
`proofs/kani/` are uncompiled templates. See `documentation/formal_verification.md`.

## Quick Start

```bash
# Build the image
docker compose -f docker/docker-compose.yml build

# Run all proofs (full verification — ~45 minutes)
docker compose -f docker/docker-compose.yml run kani

# Fast mode (fixed_equivalence only — ~5 minutes)
docker compose -f docker/docker-compose.yml run kani bash -c "./scripts/run_proofs.sh --fast"

# List all proof harnesses
docker compose -f docker/docker-compose.yml run kani bash -c "./scripts/run_proofs.sh --list"
```

## Proof Files

| # | File | Harnesses | Status |
|:---:|:---|:---:|:---|
| 1 | `kelvin-core/src/fixed_math.rs` | 8 | Compiled: add/sub/mul range, C1/C3 arithmetic facts |
| 2 | `C1_…`, `C2_…`, `C3_…`, `C4_…`, `C5_…`, `C6_…` (`proofs/kani/`) | 18 | Compiled via `kani_harnesses.rs` |
| 3 | `verify_*.rs` legacy copies (`proofs/kani/`) | 11 | Compiled via `kani_harnesses.rs` |
| 4 | `fixed_equivalence`, `acceleration_proofs`, `vec3_proofs`, `integrator_proofs`, `stability_proofs`, `extraction_proofs`, `orbital_state_proofs`, `pipeline_proofs`, `lyapunov_proofs`, `information_loss`, `fixed_safety` | — | **Not compiled** — templates |

These harnesses check arithmetic, framing and loop invariants. They do not establish the C1–C5 security claims.

## Manual Docker Commands

```bash
# Build
docker build -f docker/Dockerfile.kani -t kelvin-kani .

# Run all proofs
docker run --rm -v $(pwd):/kelvin -v kani-cargo:/root/.cargo kelvin-kani

# Run fast mode
docker run --rm -v $(pwd):/kelvin -v kani-cargo:/root/.cargo kelvin-kani --fast

# List harnesses
docker run --rm -v $(pwd):/kelvin kelvin-kani --list
```

## Resource Requirements

| Resource | Minimum | Recommended |
|:---|:---:|:---:|
| RAM | 8 GB | 32 GB |
| CPUs | 4 | 16 |
| Disk | 10 GB | 20 GB |
| Time (full) | — | ~45 minutes |
| Time (fast) | — | ~5 minutes |

## Platform Notes

- **Linux**: Native support. Kani uses CBMC which runs directly.
- **macOS**: Works via Docker Desktop. Performance is slightly slower than
  native Linux due to Linux VM overhead.
- **Windows**: Requires WSL2 backend for Docker Desktop. Use PowerShell
  with `docker compose` (not `docker-compose`).