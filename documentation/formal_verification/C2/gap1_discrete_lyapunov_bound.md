# C2 Gap 1: Discrete Lyapunov Error Bound

**Status:** Open. No formal model of the estimator.

**Claim:** the estimated discrete Lyapunov exponent satisfies

$$|\lambda_{\text{disc}}(S) - \lambda_{\text{cont}}|
  \le \frac{\varepsilon_{\text{pade}}}{S \cdot dt}
     + \varepsilon_q + \frac{\sigma\sqrt 3}{\sqrt S},$$

with $\varepsilon_{\text{pade}} \le 0.67$, $\varepsilon_q = 2^{-64}$,
$\sigma \approx 0.07$, $S = 2000$, $dt = 0.001$ yr.

## Why this is not proved

- **No model.** There is no Lean or Rust model of
  `compute_lyapunov_q32_64`; `kelvin-core/src/reference.rs` is an f64
  placeholder that returns a sum of squares.
- **The three terms are asserted.** The Padé bound, the quantization term, and
  the $\sigma/\sqrt S$ statistical term are each plausible, but none is derived
  from the code. In particular the $\sigma/\sqrt S$ term presumes i.i.d.
  samples, which a deterministic chaotic map does not provide.
- **`lyapunov_proofs.rs` is a template.** It is not in any Cargo target and has
  never been compiled.

## Why the earlier "proof" was removed

An earlier revision quoted Lean theorems `shadow_orbit_exponential_growth` and
`pade_ln_error_bound`. The first used `≈` (not Lean syntax) and a
`default {x:=0,y:=0,z:=0}` term; the second ended with `norm_num`, which cannot
prove a bound on `Real.log`. Neither theorem existed in the sources.

## What would be needed

1. A model of the estimator, including the Padé `ln` and the Q32.64 rounding.
2. A proved bound on `|ln x − pade(x)|` over the used range, plus a proof that
   the range is respected.
3. A deterministic shadowing argument (not an i.i.d. statistical one) to replace
   the $\sigma/\sqrt S$ term.

## References

- [C2 overview](readme.md)
- [C2 gap 6](gap6_lipschitz_lambda.md)
- [PROOF_INDEX.md](../PROOF_INDEX.md)
