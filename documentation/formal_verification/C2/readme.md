# C2: Lyapunov Exponent Certification

> **Status:** Open. There is no formal model of the Lyapunov estimator and no
> machine-checked theorem about it. The error budget below is a design sketch.
> **Gap documents:** `gap1`–`gap6` in this directory
> **Last updated:** 2026-11-23

---

## Design sketch (not proved)

### Kaplan–Yorke bound

$$D_{KY} = j + \frac{\sum_{k=1}^{j} \lambda_k}{|\lambda_{j+1}|}$$

The sketch asserts $\log_2 A \le D_{KY} \cdot 64$. No artifact derives this, and
no artifact ties $D_{KY}$ to a measured spectrum.

### Shadow-orbit error budget

$$\lambda_{disc} = \frac{\ln(\bar d / \delta)}{S \cdot dt}$$

$$|\lambda_{disc}(S) - \lambda_{cont}|
  \le \frac{\varepsilon_{pade}}{S \cdot dt} + \varepsilon_q
     + \frac{\sigma\sqrt{3}}{\sqrt S}$$

| Term | Value | Source |
|------|-------|--------|
| $\varepsilon_{pade}$ | $\le 0.67$ | Padé (1,1) maximum absolute error |
| $\varepsilon_q$ | $2^{-64} \approx 5.4\times10^{-20}$ | Q32.64 quantization step |
| $\sigma$ | $\approx 0.07$ | Std. dev. across 3 perturbed axes |
| $S$ | 2000 | Default shadow steps |
| $dt$ | $2^{54}$ raw $= 2^{-10}$ yr | `DEFAULT_DT` |

Each term is plausible; none is derived from a checked model.

### Spectrum

The sketch asserts the f64 QR spectrum deviates from Q32.64 by at most
$\kappa(J)\cdot\varepsilon_q$. This is a standard conditioning heuristic, not a
theorem about this implementation.

## Gap status

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | $\lvert\lambda_{disc}-\lambda_{cont}\rvert$ error bound | [`gap1_discrete_lyapunov_bound.md`](gap1_discrete_lyapunov_bound.md) | **Open** |
| 2 | Positive $\lambda$ lower bound | [`gap2_positive_lambda_certification.md`](gap2_positive_lambda_certification.md) | **Empirical** ($\lambda \approx 0.7\,\text{yr}^{-1}$) |
| 3 | Full spectrum in Q32.64 | [`gap3_q3264_lyapunov_spectrum.md`](gap3_q3264_lyapunov_spectrum.md) | **Empirical** |
| 4 | Kaplan–Yorke entropy bound | [`gap4_kaplan_yorke_entropy.md`](gap4_kaplan_yorke_entropy.md) | **Open** |
| 5 | Shadow-orbit bias constant | [`gap5_shadow_orbit_bias.md`](gap5_shadow_orbit_bias.md) | **Open** |
| 6 | Lipschitz constant of $\lambda$ | [`gap6_lipschitz_lambda.md`](gap6_lipschitz_lambda.md) | **Open** |

## Kani harnesses

`proofs/kani/lyapunov_proofs.rs` contains sketches named
`verify_pade_ln_bound`, `verify_lyapunov_division`, and
`verify_perturbation_linear_regime`. **They are templates:** the file is not
included by `kani_harnesses.rs` and has never been compiled. Do not cite them as
verified.

What *does* run is a set of arithmetic harnesses over the quantities the
estimator manipulates, in `C2_verify_lyapunov_spectrum.rs` and
`verify_lyapunov_spectrum.rs` (included by `kelvin-core/src/kani_harnesses.rs`):

| Harness | What it checks |
|---------|----------------|
| `verify_lyapunov_spectrum_comprehensive` | Padé ratio $(x-1)/(x+1) \in [0,1]$ for $x \in [1,2]$ |
| `verify_lyapunov_division` | $\lambda = \ln r / t \in [0, \ln r]$ for $t \ge 2^{54}$ |
| `verify_lyapunov_error_bounds` | quantization term is below one raw unit |
| `verify_lyapunov_quantization` | $1 / t = 0$ in raw units for $t \ge 2^{54}$ |
| `verify_shadow_orbit_bias` | bias budget is below the accumulated time |
| `verify_perturbation_in_range` | the $2^{40}$ perturbation is positive and $< 1.0$ |

None of these certifies $\lambda > 0$, the spectrum, or $D_{KY}$.

## References

- [`../../../proofs/lean/src/KelvinProofs/`](../../../proofs/lean/src/KelvinProofs/) (Lean modules — none model the Lyapunov estimator)
- `kelvin-core/src/reference.rs` (f64 placeholder)
- [`../PROOF_INDEX.md`](../PROOF_INDEX.md)

## See also

- [PROOF_INDEX.md](../PROOF_INDEX.md)
- [C2 gap documents](.)
- [Lean proofs](../../../proofs/lean/README.md)
