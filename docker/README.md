# Docker — Kani Formal Verification

This directory provides a **reproducible Docker environment** for running Kani
model-checking proof harnesses in the Kelvin cryptosystem.
The active proof suite covers L0 safety harnesses in `kelvin-core/src/fixed_math.rs`
plus the full L1–L4 harnesses in `proofs/kani/` (counts verified 2026-10-10).

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

| # | File | Harnesses | What is Proved |
|:---:|:---|:---:|:---|
| 1 | `fixed_math.rs` | 8 | L0 Safety: no panics, no overflows |
| 2 | `fixed_equivalence.rs` | 9 | L1: Q32.64 arithmetic functional equivalence |
| 3 | `acceleration_proofs.rs` | 5 | L2: Newtonian gravity invariants |
| 4 | `pipeline_proofs.rs` | 3 | L3: Pipeline integrity (domain sep, loop equiv) |
| 5 | `vec3_proofs.rs` | 18 | Vec3 vector operation proofs |
| 6 | `integrator_proofs.rs` | 9 | Verlet + Euler integrator proofs |
| 7 | `stability_proofs.rs` | 6 | Ejection + collapse detection proofs |
| 8 | `extraction_proofs.rs` | 7 | Entropy extraction safety proofs |
| 9 | `orbital_state_proofs.rs` | 8 | OrbitalState safety proofs |
| 10 | `lyapunov_proofs.rs` | 3 | Lyapunov estimation proofs |
| 11 | `information_loss.rs` | 4 | C1 information-loss bounds |

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