# C2: Lyapunov Exponent Certification — Proof Sketch

> **Results:** `proofs/kani/results/c2_validation.log`
> **Gap documents:** [`gap{1..6}_*.md`](.)

---

## Theorem 1: Kaplan-Yorke Attractor Bound

$$D_{KY} = j + \frac{\sum_{k=1}^{j} \lambda_k}{|\lambda_{j+1}|}$$

The attractor size $A$ satisfies $\log_2(A) \le D_{KY} \cdot 64$ (Q32.64 bits per dimension).

## Theorem 2: Shadow Orbit Error Budget

$$\lambda_{disc} = \frac{\ln(\bar{d} / \delta)}{S \cdot dt}$$

The estimation error is bounded by:

$$|\lambda_{disc}(S) - \lambda_{cont}| \leq \frac{\varepsilon_{pade}}{S \cdot dt} + \varepsilon_q + \frac{\sigma\sqrt{3}}{\sqrt{S}}$$

where:

| Term | Value | Source |
|------|-------|--------|
| $\varepsilon_{pade}$ | $\leq 0.67$ | Padé (1,1) max absolute error |
| $\varepsilon_q$ | $2^{-64} \approx 5.4 \times 10^{-20}$ | Q32.64 quantization step |
| $\sigma$ | $\approx 0.07$ | Std dev across 3 perturbed axes |
| $S$ | $2000$ | Default shadow steps |
| $dt$ | $2^{54}$ raw $\approx 0.0156$ yr | Verlet time step |

## Theorem 3: Full Lyapunov Spectrum

The f64 QR decomposition spectrum deviates from Q32.64 by at most $\kappa(J) \cdot \varepsilon_q$.

## All Gaps Resolved

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | $\lvert\lambda_{disc} - \lambda_{cont}\rvert$ error bound | [`gap1_discrete_lyapunov_bound`](gap1_discrete_lyapunov_bound.md) | ✅ RESOLVED |
| 2 | Positive $\lambda$ lower bound | [`gap2_positive_lambda_certification`](gap2_positive_lambda_certification.md) | ✅ RESOLVED |
| 3 | Full Lyapunov spectrum in Q32.64 | [`gap3_q3264_lyapunov_spectrum`](gap3_q3264_lyapunov_spectrum.md) | ✅ RESOLVED |
| 4 | Kaplan-Yorke entropy bound | [`gap4_kaplan_yorke_entropy`](gap4_kaplan_yorke_entropy.md) | ✅ RESOLVED |
| 5 | Shadow orbit bias constant | [`gap5_shadow_orbit_bias`](gap5_shadow_orbit_bias.md) | ✅ RESOLVED |
| 6 | Lipschitz constant of $\lambda$ | [`gap6_lipschitz_lambda`](gap6_lipschitz_lambda.md) | ✅ RESOLVED |

## Kani-Verified Harnesses

| Harness | What It Proves |
|---------|---------------|
| `verify_pade_ln_bound` | Padé approximation monotonic and non-negative |
| `verify_lyapunov_division` | $\lambda = \text{ln-ratio} / time$ finite and non-negative |
| `verify_perturbation_linear_regime` | $\delta = 2^{40}$ raw produces $\mathcal{O}(\delta)$ divergence after 1 Verlet step |