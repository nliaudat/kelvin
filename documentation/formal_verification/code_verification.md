# Code-Level Verification — L1 & L2

> **Purpose:** describe the L1 (functional equivalence) and L2 (composite
> correctness) verification strategy for the Kelvin cryptosystem.
>
> **Scope warning (2026-11-23).** The `proofs/kani/*.rs` files that are not
> included by `kelvin-core/src/kani_harnesses.rs` (for example
> `acceleration_proofs.rs`) are **templates**: they are not part of any Cargo
> target and have never been compiled. The 37 runnable harnesses are the 8 in
> `kelvin-core/src/fixed_math.rs` plus the 29 included from `proofs/kani/`. See
> [`../../docker/run-kani.sh`](../../docker/run-kani.sh) for the list.

---

## L1: Functional equivalence — dynamically scaled error bounds

Each fixed-point operation is intended to be shown equivalent to its
mathematical specification, with rounding error bounded as a function of
operand magnitude.

### Addition and subtraction

Exact for bounded inputs (no rounding), so the check is a range check:

```rust
kani::assert(result == Fixed::from_raw(a_raw + b_raw),
    "add: exact match for bounded inputs");
```

Integrated harnesses: `verify_add_no_overflow`, `verify_sub_no_overflow`.

### Multiplication

Uses high/low splitting. The integrated harness `verify_mul_range` checks the
result stays inside its documented range. Commutativity and identity are
*intended* checks (see `proofs/kani/fixed_equivalence.rs`), but that file is not
compiled.

### Division — dynamically scaled error bound

`q = num / den` rounds to within 1 ULP of `q`. Multiplying back by `den` scales
that error by `den`:

- `q * den = num - remainder`, with `remainder < den`
- the error in raw units can be as large as `|den_raw| >> 64` ULPs
- since `den_raw ≤ 8,000,000 · 2^64`, that is up to `8,000,000` ULPs

```rust
let max_error = (den.abs().to_raw() >> 64) + 2;
kani::assert(error.to_raw() <= max_error,
    "div: inverse property (result*den ≈ num, error within theoretical bound)");
```

Integrated harnesses: `verify_c1_division_remainder` (remainder range),
`verify_c1_preimage_bound`, `verify_c1_epsilon_bound` (preimage size).

**This is not an entropy statement.** Bounding a remainder does not bound
information loss; C1 remains open. See
[`C1/readme.md`](C1/readme.md).

### Square root — dynamically scaled error bound

With `r = sqrt(val)` and `|e| ≤ 1` ULP, `r² = val + 2·sqrt(val)·e + e²`. Since
`val ≤ 40,000 · 2^64`, `sqrt(val) ≤ 200 · 2^64`, so the error is at most ~400
ULPs:

```rust
let max_error = ((2 * result.to_raw()) >> 64) + 3;
kani::assert(error.to_raw() <= max_error,
    "sqrt: inverse property (sqrt(a)² ≈ a, error within theoretical bound)");
```

No integrated harness currently checks `sqrt`; the snippet above is from the
uncompiled template `proofs/kani/fixed_equivalence.rs`.

---

## L2: Composite correctness — force-based Newtonian invariants

Acceleration is verified against Newton's laws using **force-based** assertions,
because the third law relates forces (`F_01 = -F_10`), not accelerations. Since
`F = m·a`, accelerations satisfy `m0·a_01 = -m1·a_10`, and are equal in magnitude
only when `m0 = m1`.

### Properties (intended)

1. **Action–reaction (force-based):** `F_01 = -F_10`, i.e. `m1·a_01 = -m2·a_10`
2. **Direction:** acceleration of body `i` points from `i` toward `j`
3. **Single-body zero:** a lone body experiences zero acceleration
4. **Three-body symmetry:** the net force over all three bodies is zero
5. **Mass proportionality:** acceleration magnitude scales with target mass

### Where these are (and are not) checked

- **Lean:** `Gravity.pair_momentum_cancel` proves the two contributions to a
  pair's momentum cancel exactly for arbitrary masses;
  `Gravity.accel_zero_mass` proves a massless body exerts no acceleration.
- **Kani:** `proofs/kani/acceleration_proofs.rs` contains five harness sketches
  (`verify_acceleration_action_reaction`, `verify_acceleration_direction`,
  `verify_single_body_zero_acceleration`, `verify_three_body_symmetry`,
  `verify_acceleration_proportional_to_mass`). They are **templates**: not in a
  Cargo target, never compiled, never run.

To make them real, move them into `kelvin-core` under `#[cfg(kani)]`, add them
to `docker/run-kani.sh`, and run them.

---

## Proof strategy (Apple-inspired, aspirational)

Apple's corecrypto team proved assembly ≅ C ≅ FIPS specification by writing
reusable lemma libraries, proving each layer, and composing the results. Kelvin
aims for the same shape:

1. Prove each `Fixed` arithmetic op against its mathematical definition.
2. Prove `Vec3` operations compose correctly from `Fixed` ops.
3. Prove `compute_accelerations` satisfies Newton's laws via force assertions.
4. Prove the Verlet integrator preserves algebraic invariants.
5. Prove the end-to-end pipeline produces the correct keystream.
6. Derive per-step information loss from fixed-point rounding.

Steps 1–2 are partly done (Lean + three Kani harnesses). Steps 3–6 are not.

**Last updated:** 2026-11-23
