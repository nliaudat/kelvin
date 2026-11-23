# C1 Gap 3: Cumulative Loss Saturation

**Status:** Open. No proof; no formal model of entropy loss.

**Claim:** for the Verlet integrator with $N$ bodies, the total information loss
after $S$ steps is

$$L_{\text{total}}(S) = \min\bigl(S \cdot k_{\text{step}},\;
  H_{\max} - \log_2 A\bigr),$$

with $k_{\text{step}} \ge 40$ bits/step for $N=5$, $H_{\max} = 3840$ bits, and
$\log_2 A$ from the Kaplan–Yorke bound. Saturation is claimed at
$T_{\text{sat}} \approx 96$ steps.

## Status of each ingredient

| Ingredient | Status |
|------------|--------|
| $k_{\text{step}} \ge 40$ bits/step | **Open** — depends on C1 gap 1 (which is itself partial) |
| $H_{\max} = 3840$ bits | Counting only; assumes every state is reachable and equally likely |
| $\log_2 A$ from $D_{KY}$ | **Open** — see C2 gap 4 |
| The `min` formula itself | A modelling assumption, not derived |

## Why the earlier "proof" was removed

An earlier revision quoted a Lean theorem `c1_saturation_bound` containing
`by sorry` and a line `T_sat ≈ 96 [when N=5]`, which is not Lean syntax. That
theorem never existed in the sources and could not have compiled. It has been
deleted.

## What is actually proved nearby

`FixedPoint.div_rem_identity` and `FixedPoint.rem_bounds` establish that integer
division and remainder are well behaved. They say nothing about cumulative
entropy loss.

## References

- [C1 gap 1](gap1_kop_shannon_bound.md)
- [C1 gap 4](gap4_epsilon_bound.md)
- [C2 gap 4](../C2/gap4_kaplan_yorke_entropy.md)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
