# Code-Level Verification — L1 & L2 Proofs

> **Purpose:** This document describes the L1 (Functional Equivalence) and L2 (Composite Correctness) formal verification proofs for the Kelvin cryptosystem.

---

## L1: Functional Equivalence — Dynamically Scaled Error Bounds

Following Apple's methodology, each fixed-point operation is proved equivalent to its mathematical specification. Fixed-point arithmetic introduces rounding errors that must be bounded using **dynamically scaled error bounds** based on operand magnitude.

### Addition and Subtraction

For addition and subtraction, the fixed-point representation is exact for bounded inputs (no rounding):

```rust
kani::assert(result == Fixed::from_raw(a_raw + b_raw),
    "add: exact match for bounded inputs");
```

### Multiplication

Multiplication uses high/low splitting and is exact for all bounded inputs. Commutativity, identity, and zero properties are verified:

```rust
kani::assert(a * b == b * a, "mul: commutative");
kani::assert(a * Fixed::ONE == a, "mul: identity");
kani::assert(a * Fixed::ZERO == Fixed::ZERO, "mul: zero");
```

### Division — Dynamically Scaled Error Bound

Division `q = num / den` introduces a rounding error of up to 1 ULP of `q`. When multiplying `q` back by `den`, this error is scaled by `den`:

- `q * den = num - remainder`, where `remainder < den`
- Error in raw units can be up to `|den_raw| >> 64` ULPs
- Since `den_raw` can be up to `8,000,000 * AU` (i.e., `8,000,000 * 2^64`), the error can be up to 8,000,000 ULPs

```rust
let max_error = (den.abs().to_raw() >> 64) + 2;
kani::assert(error.to_raw() <= max_error,
    "div: inverse property (result*den ≈ num, error within theoretical bound)");
```

### Square Root — Dynamically Scaled Error Bound

Let `r = sqrt(val)`. The rounding error `|e| <= 1 ULP` gives `r^2 = val + 2*sqrt(val)*e + e^2`. The error `|r^2 - val|` is approximately `2 * sqrt(val) * e`.

Since `val` can be up to `40,000 * AU` (i.e., `40,000 * 2^64`), `sqrt(val)` can be up to `200 * 2^64`. The error in raw units can be up to `2 * 200 = 400` ULPs.

```rust
let max_error = ((2 * result.to_raw()) >> 64) + 3;
kani::assert(error.to_raw() <= max_error,
    "sqrt: inverse property (sqrt(a)² ≈ a, error within theoretical bound)");
```

---

## L2: Composite Correctness — Force-Based Newtonian Invariants

The acceleration computation is verified against Newton's laws using **force-based assertions** rather than acceleration-based assertions. This is critical because Newton's Third Law states that forces are equal and opposite (`F_01 = -F_10`), not accelerations (`a_01 = -a_10`).

Since `F = m * a`, the accelerations are related by `m0 * a_01 = -m1 * a_10`. They are only equal in magnitude if `m0 = m1`.

### Verified Properties

1. **Action-Reaction (Force-based)**: `F_01 = -F_10` via `m1*a_01 = -m2*a_10`
2. **Direction**: Acceleration of body i points from body i toward body j
3. **Single-body zero**: A body alone experiences zero acceleration
4. **Three-body symmetry**: Net force on all three bodies sums to zero
5. **Mass proportionality**: Acceleration magnitude scales with target mass

### Proof Harnesses

Five proof harnesses in `proofs/kani/acceleration_proofs.rs`:

| Harness | Property | Assertion |
|---------|----------|-----------|
| `verify_action_reaction` | Newton's Third Law | `m1*a_01 == -m2*a_10` (force-based) |
| `verify_acceleration_direction` | Direction | `a_ij` points from i toward j |
| `verify_single_body_zero` | No self-interaction | Single body has zero acceleration |
| `verify_three_body_symmetry` | Net force zero | `Σ m_i * a_i == 0` |
| `verify_mass_proportionality` | Mass scaling | `|a_01| / |a_10| == m2 / m1` |

---

## Proof Strategy (Apple-Inspired)

Apple's corecrypto team proved that their ARM64 assembly is equivalent to their C code, and their C code is equivalent to the FIPS mathematical specification. They achieved this by:
1. **Writing reusable lemma libraries** — common proof patterns for operations
2. **Proving equivalence at each level** — assembly ≅ C ≅ spec
3. **Composing proofs** — building complex proofs from verified components

Kelvin adapts this strategy for Rust + Kani:
1. **Prove each Fixed arithmetic op** against its mathematical definition
2. **Prove Vec3 operations** compose correctly from Fixed ops
3. **Prove `compute_accelerations`** satisfies Newton's laws via force-based assertions
4. **Prove Verlet integrator** preserves algebraic invariants (momentum conservation)
5. **Prove end-to-end pipeline** produces correct keystream
6. **Prove per-step information loss** from fixed-point rounding establishes irreversibility