# Formal Verification — Kelvin Cryptosystem

## Overview

This document describes the formal verification strategy for the Kelvin
cryptosystem, following the blueprint established by Apple's corecrypto
formal verification methodology.

Apple's approach proves **functional equivalence** at every level of the stack:
mathematical specification → C implementation → ARM64 assembly. Kelvin adapts
this blueprint for a pure-Rust codebase using the Kani Rust Verifier.

## Proof Architecture

```
Mathematical Specification (proofs/specs/)
    ↓ equivalence (Kani)
Fixed-point Q32.64 Arithmetic (kelvin-core/src/fixed_math.rs)
    ↓ equivalence (Kani)
Vec3 Vector Operations (kelvin-core/src/body.rs)
    ↓ equivalence (Kani)
compute_accelerations (kelvin-core/src/integrator.rs)
    ↓ equivalence (Kani)
Verlet/Euler Integrator (kelvin-core/src/integrator.rs)
    ↓ equivalence (Kani)
simulate() + extract_seed() Pipeline
    ↓ equivalence (golden hash)
End-to-End Keystream Output
```

## Proof Levels

| Level | What is Proved | Method | Location |
|-------|---------------|--------|----------|
| **L0: Safety** | No panics, no overflows under bounded inputs | Kani model checking | `fixed_math.rs` (existing) |
| **L1: Functional Equivalence** | Arithmetic ops match mathematical spec within dynamically scaled error bounds | Kani with reference computation | `proofs/kani/fixed_equivalence.rs` |
| **L2: Composite Correctness** | `compute_accelerations` satisfies Newton's laws via force-based assertions | Kani with Newtonian invariants | `proofs/kani/acceleration_proofs.rs` |
| **L3: Pipeline Integrity** | Full `simulate_and_extract_seed` produces correct output | Kani + golden hash | `proofs/kani/pipeline_proofs.rs` |
| **L4: Determinism** | Bit-identical results across platforms | Integration tests | `tests/kelvin_tests/determinism.rs` |

## Proof Strategy (Apple-Inspired)

Apple's corecrypto team proved that their ARM64 assembly is equivalent to their
C code, and their C code is equivalent to the FIPS mathematical specification.
They achieved this by:

1. **Writing reusable lemma libraries** — common proof patterns for operations
   like polynomial addition loops vs. abstract map operations.
2. **Proving equivalence at each level** — not trying to prove assembly directly
   against the FIPS spec, but proving assembly ≅ C ≅ spec.
3. **Composing proofs** — building complex proofs from verified components.

Kelvin adapts this strategy for Rust + Kani:

1. **Prove each Fixed arithmetic op** against its mathematical definition
   (e.g., `Fixed::add(a,b) == a + b` for all bounded inputs).
2. **Prove Vec3 operations** compose correctly from Fixed ops.
3. **Prove `compute_accelerations`** satisfies Newton's laws via force-based
   assertions (`F_01 = -F_10` using `m1*a_01 = -m2*a_10`), direction
   (acceleration points from body i to body j), and mass proportionality.
4. **Prove Verlet integrator** preserves algebraic invariants (momentum conservation).
5. **Prove end-to-end pipeline** produces correct keystream.

## L1: Functional Equivalence — Dynamically Scaled Error Bounds

Following Apple's methodology, each fixed-point operation is proved equivalent
to its mathematical specification. However, fixed-point arithmetic introduces
rounding errors that must be bounded. The key insight is that **static error
bounds fail under Kani model checking for large inputs**, so we use dynamically
scaled error bounds based on the magnitude of the operands.

### Addition and Subtraction

For addition and subtraction, the fixed-point representation is exact for
bounded inputs (no rounding):

```rust
kani::assert(result == Fixed::from_raw(a_raw + b_raw),
    "add: exact match for bounded inputs");
```

### Multiplication

Multiplication uses high/low splitting and is exact for all bounded inputs.
Commutativity, identity, and zero properties are verified:

```rust
kani::assert(a * b == b * a, "mul: commutative");
kani::assert(a * Fixed::ONE == a, "mul: identity");
kani::assert(a * Fixed::ZERO == Fixed::ZERO, "mul: zero");
```

### Division — Dynamically Scaled Error Bound

Division `q = num / den` introduces a rounding error of up to 1 ULP of `q`.
When multiplying `q` back by `den`, this error is scaled by `den`:

- `q * den = num - remainder`, where `remainder < den`
- Error in raw units can be up to `|den_raw| >> 64` ULPs
- Since `den_raw` can be up to `8,000,000 * AU` (i.e., `8,000,000 * 2^64`),
  the error can be up to 8,000,000 ULPs

The dynamically scaled error bound:

```rust
let max_error = (den.abs().to_raw() >> 64) + 2;
kani::assert(error.to_raw() <= max_error,
    "div: inverse property (result*den ≈ num, error within theoretical bound)");
```

### Square Root — Dynamically Scaled Error Bound

Let `r = sqrt(val)`. The rounding error of `r` is up to 1 ULP of `r`, so
`r = sqrt(val) + e` where `|e| <= 1 ULP`. Then `r^2 = val + 2*sqrt(val)*e + e^2`.
The error `|r^2 - val|` is approximately `2 * sqrt(val) * e`.

Since `val` can be up to `40,000 * AU` (i.e., `40,000 * 2^64`), `sqrt(val)` can
be up to `200 * 2^64`. Therefore, the error in raw units can be up to
`2 * 200 = 400` ULPs.

The dynamically scaled error bound:

```rust
let max_error = ((2 * result.to_raw()) >> 64) + 3;
kani::assert(error.to_raw() <= max_error,
    "sqrt: inverse property (sqrt(a)² ≈ a, error within theoretical bound)");
```

## L2: Composite Correctness — Force-Based Newtonian Invariants

The acceleration computation is verified against Newton's laws using
**force-based assertions** rather than acceleration-based assertions.
This is critical because Newton's Third Law states that forces are equal
and opposite (`F_01 = -F_10`), not accelerations (`a_01 = -a_10`).

Since `F = m * a`, the accelerations are related by `m0 * a_01 = -m1 * a_10`.
They are only equal in magnitude if the masses of the two bodies are equal
(`m0 = m1`). Because `m1_raw` and `m2_raw` are symbolic variables that can
be different, an acceleration-based assertion would fail under Kani model
checking.

### Verified Properties

1. **Action-Reaction (Force-based)**: `F_01 = -F_10` via `m1*a_01 = -m2*a_10`
2. **Direction**: Acceleration of body i points from body i toward body j
3. **Single-body zero**: A body alone experiences zero acceleration
4. **Three-body symmetry**: Net force on all three bodies sums to zero
5. **Mass proportionality**: Acceleration magnitude scales with target mass

### Proof Harnesses

Five proof harnesses are implemented in `proofs/kani/acceleration_proofs.rs`:

| Harness | Property | Assertion |
|---------|----------|-----------|
| `verify_action_reaction` | Newton's Third Law | `m1*a_01 == -m2*a_10` (force-based) |
| `verify_acceleration_direction` | Direction | `a_ij` points from i toward j |
| `verify_single_body_zero` | No self-interaction | Single body has zero acceleration |
| `verify_three_body_symmetry` | Net force zero | `Σ m_i * a_i == 0` |
| `verify_mass_proportionality` | Mass scaling | `|a_01| / |a_10| == m2 / m1` |

## Running the Proofs

```bash
# Run all Kani proofs (requires Kani installed)
cargo kani -p kelvin-core

# Run determinism integration tests
cargo test -p kelvin --test full_pipeline

# Run constant-time verification
cargo run -p constant_time_bench
```

## References

- Apple Security Research (2026). "Formal verification of corecrypto for
  post-quantum cryptography." [security.apple.com/blog/formal-verification-corecrypto](https://security.apple.com/blog/formal-verification-corecrypto/)
- Apple Inc. (2026). corecrypto open source release.
  [github.com/apple/corecrypto](https://github.com/apple/corecrypto)
- Kani Rust Verifier. [model-checking.github.io/kani/](https://model-checking.github.io/kani/)
