# C5 Gap 3: Stability Reduction Factor — RE-OPENED (2026-10-10)

> **Status:** ❌ RE-OPENED — the §2.3 argument is inverted (proves valid
> fraction ≤ 2⁻¹⁰⁰, opposite of the theorem) and contradicted by the cited
> Monte Carlo (>90% pass). The 2⁻¹⁰⁰ reduction factor is unsupported.
> Candidate replacement: rejection rate of `validate()` over keygen output
> (≈ 1, i.e. negligible reduction).
> **Date:** 2026-05-31 (re-opened 2026-10-10)
> **Conjecture:** C5 — Valid Configuration Space Cardinality

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
