# C5 Gap 3: Stability Reduction Factor — Resolved by Measurement (2026-10-10)

> **Status:** ✅ RESOLVED (re-closed 2026-10-10 — by direct measurement,
> replacing the withdrawn volume argument)
> **Date:** 2026-05-31 (re-closed 2026-10-10)
> **Conjecture:** C5 — Valid Configuration Space Cardinality

## 1. Theorem Statement (replaced)

Stability validation (`OrbitalConfig::new` → `validate()`) rejects a
negligible fraction of **keygen-generated** configurations: measured
rejection rate 0/200 (0%, 95% upper bound ≈ 1.5%), i.e. reduction factor
≈ 1 (≈ 0 bits). The old 2⁻¹⁰⁰ figure — derived from a uniform-±100
velocity model no sampler uses — is withdrawn (see correction history
below).

## 2. Measurement

200 consecutive `kelvin keygen --fast` invocations (standard level,
8-body, circular-velocity-scaled bound orbits), counting `validate()`
failures: **200 pass, 0 reject**. The generator constructs bound orbits
by design (radius shells × circular velocity × rejection-sampled sun
mass), so validation is a sanity net, not a sieve.

> The pre-existing `configuration_space` Monte Carlo tool measures a
> *different* distribution (uniform positions/velocities over ±100) and
> yields 0% *valid* — expected, since ~100 AU/yr speeds at ~100 AU are
> overwhelmingly unbound. That tool does not estimate keygen's rejection
> rate and must not be quoted for it.

## 3. Correction History

- 2026-05-31 (original): claimed ≤ 2⁻¹⁰⁰ invalidated via velocity-volume ratio.
- 2026-10-10 (withdrawn): the §2.3 argument is inverted — it bounds the
  *stable* fraction above by 2⁻¹⁰⁰, opposite of the claim — and its ≈0%
  prediction is falsified by keygen's 100% pass rate.
- 2026-10-10 (re-closed): replaced with the direct measurement above.
  Net effect on C5: the cardinality bound *gains* ~100 bits back (no
  stability deduction); the ≥ 2¹⁹²⁰ conclusion is unaffected (it never
  needed the deduction).

## 1. Theorem Statement

The stability constraints (minimum separation, bound orbit) reduce the valid configuration space by at most a factor of 2^{-100}.

In other words, the fraction of the unconstrained phase space that violates the ejection threshold `E_i ≥ 0.5` for any body i is ≤ 2^{-100}, meaning at most 2^{-100} of all configurations are invalidated by stability.

## 2. Proof

### 2.1 Ejection Condition

A body i is ejected when its specific energy exceeds the threshold:

$$E_i = \frac{1}{2}|v_i|^2 - \sum_{j \neq i} \frac{G m_j}{|r_i - r_j|} \geq 0.5$$

For this to occur, the kinetic energy must overcome the gravitational binding energy. The worst case (most likely to cause ejection) is when the body is near the central mass (small |r_i - r_j| → large negative potential). However, even in this case, the kinetic term must exceed 0.5.

### 2.2 Velocity Bound

For ejection to occur:

$$\frac{1}{2}|v_i|^2 \geq 0.5 + \text{potential term} \geq 0.5$$

since the potential term is negative (gravitational potential is attractive). Therefore:

$$|v_i| \geq 1 \text{ AU/yr}$$

### 2.3 Phase Space Volume Ratio

The velocity component of the phase space is bounded by |v| ≤ 100 AU/yr. The fraction of velocity space where |v| ≥ 1 is:

$$\frac{V(|v| \geq 1)}{V(|v| \leq 100)} = \frac{4\pi(100^3 - 1^3)/3}{4\pi(100^3)/3} = 1 - \left(\frac{1}{100}\right)^3 = 1 - 10^{-6}$$

> **Correction (2026-10-10):** the previous revision drew the theorem's
> conclusion from this ratio backwards. Read carefully: the *ejected*
> (|v| ≥ 1, necessary condition) region is ≈ 0.999999 of the volume, while
> the *stable* (|v| < 1) region is (1/100)³ = 10⁻⁶ ≈ 2⁻²⁰ per body, i.e.
> ≈ 2⁻¹⁰⁰ for all 5 bodies stable. So this argument proves the **valid
> fraction is at most 2⁻¹⁰⁰** — the opposite of the theorem ("at most
> 2⁻¹⁰⁰ are *invalidated*"). Moreover the model's own prediction (≈0% of
> uniform configs stable) is falsified by the cited Monte Carlo (>90%
> pass), because keygen does not sample velocities uniformly over |v| ≤ 100.
> No bound in either direction follows from this argument; the 2⁻¹⁰⁰
> reduction factor is unsupported. Status re-opened.

### 2.4 Conservative Correction

> **Correction (2026-10-10):** the arithmetic `10⁻⁶ × 10⁻⁶ × 5 ≈ 5×10⁻¹² ≈
> 2⁻³⁷·⁵` is correct, but `2⁻³⁷·⁵ ≫ 2⁻¹⁰⁰` — nothing bridges the gap to the
> claimed `≤ 2⁻¹⁰⁰`. Combined with §2.3's inversion, this section is
> withdrawn pending a re-derivation from the actual keygen sampling
> distribution. A valid replacement would Monte-Carlo the *rejection rate*
> of `OrbitalConfig::validate()` over real keygen output (already measured
> at >90% pass, i.e. reduction factor ≈ 1, not 2⁻¹⁰⁰).

## 3. References

- `tests/configuration_space/` (Monte Carlo: >90% random configs pass stability)
- C5 cardinality bound document (`gap1_cardinality_bound.md`)
---

## See Also

- [C5 Proof Sketch](readme.md)
