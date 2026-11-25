# Docker — verification and analysis environments

Three independent setups:

| Setup | Compose file | Contents |
|-------|--------------|----------|
| Kani + cross | [`docker-compose.yml`](docker-compose.yml) | Rust model checking, multi-arch builds |
| Formal / analysis tools | [`docker-compose.formal.yml`](docker-compose.formal.yml) | ProVerif, Tamarin, CryptoVerif, EasyCrypt, Z3/CVC5, Rust analysis |
| Details | [`formal/README.md`](formal/README.md) | Tool images, models, usage |

## Kani

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani            # all 37
docker compose -f docker/docker-compose.yml run --rm kani --proof verify_add_no_overflow
```

37 harnesses are compiled into `kelvin-core`:

- **8** in `kelvin-core/src/fixed_math.rs` under `#[cfg(kani)]`
  (`verify_add_no_overflow`, `verify_sub_no_overflow`, `verify_mul_range`,
  `verify_c1_preimage_bound`, `verify_c1_epsilon_bound`,
  `verify_c1_division_remainder`, `verify_c3_step_damping`,
  `verify_c3_two_step_dispersion`);
- **29** in `proofs/kani/*.rs`, included by
  `kelvin-core/src/kani_harnesses.rs` under `#[cfg(kani)]`.

These harnesses check arithmetic, framing and loop invariants. They do not establish the C1–C5 security claims.

The other `proofs/kani/*.rs` files (for example `fixed_equivalence.rs`,
`pipeline_proofs.rs`, `acceleration_proofs.rs`) are **templates**: they are not
in any Cargo target and are never compiled. Do not describe them as verified.

What the harnesses do and do not constrain is documented in
[`../documentation/formal_verification/HARNESS_REVIEW.md`](../documentation/formal_verification/HARNESS_REVIEW.md).

## Formal and analysis tools

```bash
docker compose -f docker/docker-compose.formal.yml build
docker compose -f docker/docker-compose.formal.yml run --rm proverif formal/proverif/kelvin_smoke.pv
docker compose -f docker/docker-compose.formal.yml run --rm tamarin --prove formal/tamarin/kelvin_smoke.spthy
```

See [`formal/README.md`](formal/README.md) for the service table, image names
and the Rust analysis tooling (`cargo-fuzz`, `miri`, `cargo-audit`,
`cargo-deny`, `cargo-mutants`).

## Resource requirements

| Resource | Kani | Formal tools |
|:---|:---:|:---:|
| RAM | 8 GB min, 32 GB recommended | 4 GB |
| CPUs | 4 min, 16 recommended | 2 |
| Disk | 10 GB | 6 GB (EasyCrypt is the largest) |

## Platform notes

- **Linux**: native support.
- **macOS**: via Docker Desktop; slower due to the Linux VM.
- **Windows**: Docker Desktop with WSL2. Use PowerShell and `docker compose`
  (not `docker-compose`).
