# C1 Gap 7: Verlet Double-Computation Effect

> **Machine-checked status:** see [PROOF_INDEX.md](../PROOF_INDEX.md). The label
> below states what is actually established. The surrounding text is the written
> argument; it is not a proof unless the label says so.

> **Status:** Partial. The Verlet two-kick schedule is proved in Lean (`Integrator.lean`); the op-count accounting is not.

> **Date:** 2026-05-30
> **Conjecture:** C1 — Fixed-Point Information Loss (Irreversibility)
> **Kani Cross-Reference:** `verify_c1_rounding_op_count` (templated), code inspection of `kelvin-core/src/integrator.rs`

---

## 1. Statement

**Claim:** The Verlet integrator's double evaluation of the acceleration (kick-drift-kick) correctly doubles the number of rounding operations per step, and this doubling is accounted for in the C1 per-step loss count `k_step = 2N(N−1)` without introducing unexpected extra errors or artifacts.

---

## 2. Verification

### 2.1 Code Inspection

The Verlet step implementation in `kelvin-core/src/integrator.rs` lines 124–147:

```rust
pub fn verlet_step(bodies: &mut [OrbitalBody], dt: Fixed, softening: Fixed, g: Fixed) {
    let half_dt = dt / Fixed::from_int(2);

    // Step 1: Kick (half step) — calls compute_accelerations
    let accelerations = compute_accelerations(bodies, softening, g);
    for (body, acc) in bodies.iter_mut().zip(accelerations.iter()) {
        body.velocity += acc.scale(half_dt);
    }

    // Step 2: Drift (full step) — exact multiplication and addition
    for body in bodies.iter_mut() {
        body.position += body.velocity.scale(dt);
    }

    // Step 3: Compute new accelerations — calls compute_accelerations AGAIN
    let new_accelerations = compute_accelerations(bodies, softening, g);

    // Step 4: Kick (half step)
    for (body, acc) in bodies.iter_mut().zip(new_accelerations.iter()) {
        body.velocity += acc.scale(half_dt);
    }
}
```

Each call to `compute_accelerations`:
- Iterates over all `N(N−1)/2` pairs
- Per **1 square root** (`dist_sq.sqrt()`) per pair
- Per **1 division** (`g / dist_cubed`) per pair

Two acceleration calls → each pair contributes **4 rounding operations** per Verlet step (2 sqrt + 2 div).

### 2.2 What is and is not checked

There is no runnable harness that counts rounding operations. The file
`proofs/kani/information_loss.rs` mentions `verify_c3_rounding_op_count`, but it
is a template: it is not included by `kelvin-core/src/kani_harnesses.rs` and has
never been compiled.

What *is* machine-checked is the schedule, in
`proofs/lean/src/KelvinProofs/Integrator.lean`: `step_fst` shows the drift uses
the post-kick velocity, and `step_snd` shows both kicks use the half timestep.
The operation count itself is by inspection of `compute_accelerations`, and the
conversion from operations to bits is not proved.

For N=2: 1 pair, 2 acceleration calls, 2 rounding operations per pair per call,
so 4 rounding operations per step. This matches the formula
`k_step = (N(N−1)/2) × 2 × 2 = 2N(N−1)` *as a count of operations*.

### 2.3 Euler Comparison

The explicit Euler integrator (`kelvin-core/src/integrator.rs` lines 98–116) computes acceleration **once** per step, confirming the Verlet double-computation is explicit and intentional:

```rust
pub fn euler_step(bodies: &mut [OrbitalBody], dt: Fixed, softening: Fixed, g: Fixed) {
    // Step 1: Compute accelerations (ONCE)
    let acc = compute_accelerations(bodies, softening, g);
    // Step 2: Update positions with OLD velocity
    // Step 3: Update velocities
}
```

| Integrator | Accelerations/Step | Rounding Ops/Step | `k_step` (N=5) |
|------------|-------------------|-------------------|--------------|
| Verlet | 2 | `4 × N(N−1)/2 = 2N(N−1)` | 40 bits |
| Euler | 1 | `2 × N(N−1)/2 = N(N−1)` | 20 bits |

The Euler counting provides a cross-check: Verlet has exactly **twice** the rounding operations of Euler per step, consistent with the two acceleration calls.

### 2.4 No Additional Error Sources — CORRECTED (2026-10-10)

> The previous revision of this table labeled the kick/drift arithmetic
> "exact". That is false against the implementation
> (`kelvin-core/src/integrator.rs:127-145`): `dt / Fixed::from_int(2)` is
> a full fixed-point `Div` (≤1 ULP rounding, not an exact halve), and
> `Vec3::scale`/`+=` are truncating `Mul`/`Add` (≤1–2 ULP each, may wrap).
> Corrected classification:

| Operation | Nature | Counted? |
|-----------|--------|----------|
| `dt / 2` (division) | Rounding division (≤1 ULP) | ❌ NOT in the `2N(N−1)` formula — a 41st op/step for N=5 |
| `acc.scale(half_dt)` | Rounding multiplication | ❌ Uncounted (2N per step) |
| `velocity += …`, `position += …` | Rounding addition | ❌ Uncounted (2N per step) |
| `compute_accelerations` | Contains sqrt + div | ✅ Counted (2 calls) |

The `k_step = 40` figure therefore counts only the acceleration-evaluation
rounding ops; kick/drift arithmetic adds further (smaller) rounding. The
40-bit/step loss rate is retained as an approximate lower bound, not an
exact op census.

---

## 3. Confirmed Counting Formula

The per-step rounding operation count for the Verlet integrator:

$$k_{\text{step}} = (\text{pairs}) \times (\text{ops/pair}) \times (\text{accel calls})$$

$$k_{\text{step}} = \frac{N(N-1)}{2} \times 2 \times 2 = 2N(N-1)$$

For N=5:

$$k_{\text{step}} = 2 \times 5 \times 4 = 40 \text{ rounding operations per step}$$

Each rounding operation contributes at least `k_op ≥ 1` bit of information loss (Gaps 1, 4), giving a per-step loss of at least `k_step × k_op ≥ 40 × 0.980 ≈ 39.2 bits`.

---

## 4. References

1. `kelvin-core/src/integrator.rs` lines 124–147 (Verlet step implementation)
2. `kelvin-core/src/integrator.rs` lines 98–116 (Euler step implementation for comparison)
3. `proofs/kani/information_loss.rs` lines 207–244 (`verify_c1_rounding_op_count` harness)
4. `documentation/formal_verification/C1/gap1_kop_shannon_bound.md` (k_op ≥ 1 bit)

---

## See Also

- [C1 Proof Sketch](readme.md)
