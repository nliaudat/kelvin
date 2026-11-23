# C2 Gap 4: Kaplan–Yorke Entropy Bound

**Status:** Open. No proof; the earlier "theorem" was vacuous.

**Claim:** for an $n$-dimensional system with Lyapunov exponents
$\lambda_1 \ge \dots \ge \lambda_n$,

$$D_{KY} = j + \frac{\sum_{i=1}^{j}\lambda_i}{|\lambda_{j+1}|},$$

where $j$ is the largest index with $\sum_{i\le j}\lambda_i \ge 0$. The sketch
asserts that the attractor satisfies $\log_2 A \le D_{KY}\cdot 64$, and for the
8-body system $D_{KY} \in [3,4]$, hence $\log_2 A \le 256$.

## Why this is not proved

1. **No spectrum is an input.** The Kaplan–Yorke formula takes the Lyapunov
   spectrum. The claimed theorem took no spectrum, so it could not have said
   anything about Kelvin's attractor.
2. **The earlier statement was vacuous.** It was
   `∃ D_KY, D_KY ≤ 4 ∧ D_KY > 3`, discharged with `use 3.5`. That proves only
   that some number lies in $(3,4]$. It has no connection to the dynamics.
3. **`D_KY ∈ [3,4]` is an empirical estimate** for chaotic $n$-body systems, not
   a theorem about this simulator.
4. **"Each dimension contributes 64 bits" is a modelling choice**, not a
   consequence of $D_{KY}$. Converting a fractal dimension into a bit bound
   requires a covering argument with an explicit metric.

## What a real theorem would look like

```
theorem ky_bound (spectrum : List ℝ)
    (h_sorted : spectrum.Sorted (· ≥ ·))
    (h_sum_pos : 0 < (spectrum.take j).sum) ... :
    log2 (attractorVolume spectrum) ≤ kyDimension spectrum * 64
```

with `kyDimension` defined from the spectrum, and `attractorVolume` tied to the
simulator. Neither definition exists.

## Related

- [C2 gap 1](gap1_discrete_lyapunov_bound.md) — estimator error bound (open)
- [C2 gap 3](gap3_q3264_lyapunov_spectrum.md) — spectrum measurement (empirical)
- [C1 gap 3](../C1/gap3_saturation_bound.md) — uses $\log_2 A$ (open)
- [PROOF_INDEX.md](../PROOF_INDEX.md)

## Reference

Kaplan, J. L., & Yorke, J. A. (1979). "Chaotic behavior of multidimensional
difference equations." In *Functional Differential Equations and Approximation
of Fixed Points*, Springer LNM 730, 204–227.
