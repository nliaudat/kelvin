# Kani Proof Architecture — Kelvin Cryptosystem

## Proof Levels (Apple corecrypto Blueprint)

Kelvin follows Apple's formal verification methodology with 5 levels of proof
increasing in complexity and scope:

```
L0: Safety          ──  No panics, no overflows under bounded inputs
L1: Functional Eq   ──  Arithmetic ops match mathematical spec
L2: Composite       ──  Newton's laws satisfied by compute_accelerations
L3: Pipeline        ──  Full simulate_and_extract_seed produces correct output
L4: Determinism     ──  Bit-identical results across platforms
```

## Proof Harness Details

### L0: Safety (`kelvin-core/src/fixed_math.rs` — 3 harnesses)

These harnesses confirm that Q32.64 fixed-point arithmetic never panics
or overflows under physically-realistic orbital bounds.

| Harness | Property | Bound |
|:---|:---|---:|
| `verify_add_no_overflow` | add in [-100, 100] AU | [-200, 200] AU |
| `verify_sub_no_overflow` | sub in [-100, 100] AU | [-200, 200] AU |
| `verify_mul_range` | mul in [-4, 4] AU | [-16, 16] AU² |

### L1: Fixed-Point Equivalence (`proofs/kani/fixed_equivalence.rs` — 5 harnesses)

| Harness | Property | Error Bound |
|:---|:---|---:|
| `verify_add_functional_equivalence` | a + b = a_raw + b_raw | Exact (0 ULP) |
| `verify_sub_functional_equivalence` | a - b = a_raw - b_raw | Exact (0 ULP) |
| `verify_mul_commutative` | a * b = b * a | Exact |
| `verify_mul_identity` | a * 1 = a | Exact |
| `verify_mul_zero` | a * 0 = 0 | Exact |
| `verify_div_inverse` | (a/b) * b ≈ a | |den_raw| >> 64 + 2 |
| `verify_sqrt_inverse` | sqrt(a)² ≈ a | (2 * result_raw) >> 64 + 3 |

### L2: Composite Correctness (`proofs/kani/acceleration_proofs.rs` — 5 harnesses)

| Harness | Physical Law |
|:---|:---|
| `verify_action_reaction` | Newton's Third Law: F_01 = -F_10 within 2 ULP |
| `verify_acceleration_direction` | a_ij points from body i toward body j |
| `verify_single_body_zero` | Single body experiences zero acceleration |
| `verify_three_body_symmetry` | Net force on symmetric configuration is zero |
| `verify_mass_proportionality` | |a| ∝ source mass |

### L3: Pipeline Integrity (`proofs/kani/pipeline_proofs.rs` — 3 harnesses)

| Harness | What it proves |
|:---|:---|
| `verify_pipeline_invariants` | Momentum conservation, no collapse, no ejection for 2-body, 10 steps |
| `verify_extract_domain_sep_symbolic` | SHAKE256 extraction with distinct domain separators |
| `verify_simulate_loop_equivalence` | simulate(N) = verlet_step() × N (loop unrolling equivalence) |

## Running Proofs in Docker

```bash
# Full suite — all active harnesses (~45 min)
docker compose -f docker/docker-compose.yml run kani

# Fast mode — L0 + L1 only (~5 min)
docker compose -f docker/docker-compose.yml run --entrypoint "run-kani --fast" kani

# Individual proof file (example)
docker compose -f docker/docker-compose.yml run --entrypoint "bash -c 'cargo kani -p kelvin-core --harness verify_pipeline_invariants --enable-unstable --restrict-vtable'" kani
```

## Debugging Proof Failures

If a proof fails, the Docker container will output the failing harness name
and the specific assertion that failed. Example:

```
$ docker compose -f docker/docker-compose.yml run kani
...
[1/8] fixed_equivalence.rs — Q32.64 arithmetic functional equivalence
** 1 of 5 failed
Failed: verify_mul_commutative
  - Assertion "mul: commutative (a*b == b*a)" failed
```

Common failure modes:
1. **UB overflow**: An input constraint is too permissive
2. **Rounding error**: The error bound assertion is too tight
3. **Unwinding**: Need to increase `--default-unwind` for loop-heavy harnesses

## References

- Apple Security Research (2026). "Formal verification of corecrypto for
  post-quantum cryptography."
- Kani Rust Verifier: https://model-checking.github.io/kani/
- CBMC: https://www.cprover.org/cbmc/