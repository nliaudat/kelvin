# C1: Fixed-Point Information Loss

> **Status:** Partial. Two of the seven gaps have a machine-checked *related*
> fact; none of the entropy claims are proved.
> **Gap documents:** `gap1`–`gap7` in this directory
> **Last updated:** 2026-11-23

---

## Motivation

The L1/L2 work verifies that each fixed-point operation produces a
mathematically correct result within a bounded error. Functional equivalence is
**not** sufficient for cryptographic security. C1 tries to quantify
irreversibility: each rounding operation discards state information, so the
forward map should be many-to-one.

The argument below is a *sketch*. It is not a proof, and the current Lean/Kani
artifacts do not establish it.

## State space

$$X = (\mathbb{Z}/2^{128})^{6N}$$

Each of the $6N$ state components is a 128-bit `i128` in raw fixed-point
representation. If every state were reachable and equally likely, the state
space would hold

$$H_{\max} = \log_2 |X| = 6N \times 128 \text{ bits},$$

i.e. $3840$ bits for $N=5$. Reachability and uniformity are **not** established.

## Rounding operations per step

The Verlet integrator evaluates accelerations twice per step
(kick-drift-kick):

```rust
let diff = bodies[j].position - bodies[i].position;  // exact
let dist_sq = diff.length_squared() + softening_sq;  // exact
let dist = dist_sq.sqrt();                           // 1 rounding op
let dist_cubed = dist_sq * dist;                     // exact
let factor = g / dist_cubed;                         // 1 rounding op
```

| Integrator | Ops per pair | Pairs | Total ops | For N=5 |
|------------|--------------|-------|-----------|---------|
| Verlet | $2 \times 2 = 4$ | $N(N-1)/2$ | $2N(N-1)$ | 40 |
| Euler | $1 \times 2 = 2$ | $N(N-1)/2$ | $N(N-1)$ | 20 |

## The claimed per-operation loss

The division discards the remainder
$r = (g_{\text{raw}} \cdot 2^{64}) \bmod d^3_{\text{raw}}$, which depends on the
low bits of $d^3$. The sketch argues that for uniformly distributed input this
carries 1 bit of Shannon entropy, giving $k_{op} \ge 1$ (with an
$\varepsilon$-bound $k_{op} \ge 0.98$).

**What is actually proved:** `FixedPoint.div_rem_identity`
($a / b \cdot b + a \% b = a$) and `FixedPoint.rem_bounds`
($0 \le a \% b < b$). These are identities about the remainder. They say nothing
about entropy, uniformity, or independence between operations.

## Claimed per-step and saturation model

$$k_{\text{step}} = 2N(N-1) \times k_{\text{op}} \ge 2N(N-1) \text{ bits},$$

$$L_{\text{total}}(S) = \min(S \cdot k_{\text{step}},\; H_{\max} - \log_2 A).$$

For $N=5$ the sketch gives $k_{\text{step}} \ge 40$ bits and saturation at
$T_{\text{sat}} \approx 96$ steps. Neither number has a checked derivation.

## Gap status

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | $k_{op} \ge 1$ bit | [`gap1_kop_shannon_bound.md`](gap1_kop_shannon_bound.md) | **Partial** — division identity only |
| 2 | Uniformity of $d^3$ | [`gap2_uniform_distribution.md`](gap2_uniform_distribution.md) | **Empirical** |
| 3 | Cumulative loss saturation | [`gap3_saturation_bound.md`](gap3_saturation_bound.md) | **Open** |
| 4 | $\varepsilon$-bound $k_{op} \ge 1-\varepsilon$ | [`gap4_epsilon_bound.md`](gap4_epsilon_bound.md) | **Partial** — Kani arithmetic check |
| 5 | Rounding error independence (Lemma A3) | [`gap5_error_independence.md`](gap5_error_independence.md) | **Empirical / assumption** |
| 6 | Hartley vs min-entropy | [`gap6_hartley_min_entropy.md`](gap6_hartley_min_entropy.md) | **Open** |
| 7 | Verlet double-computation | [`gap7_verlet_double_effect.md`](gap7_verlet_double_effect.md) | **Partial** — schedule modelled |

## Kani harnesses (integrated)

Three run under `#[cfg(kani)]` in `kelvin-core/src/fixed_math.rs`, and five more
are included from `proofs/kani/` by `kelvin-core/src/kani_harnesses.rs`:

| Harness | What it checks |
|---------|----------------|
| `verify_c1_preimage_bound` | local preimage size at the softening limit |
| `verify_c1_epsilon_bound` | preimage bound for the implemented division |
| `verify_c1_division_remainder` | $0 \le r < \text{den}$ |
| `verify_information_loss_total` | model loss never exceeds $H_{\max}$ |
| `verify_information_loss_monotone` | model loss is monotone in $S$ |
| `verify_saturation_point` | 96 steps saturate the $N=5$ model ($96 \times 40 = 3840$) |
| `verify_information_loss_saturating` | capped loss is the cap or the uncapped value |
| `verify_information_loss_cap_boundary` | larger per-step loss saturates no later |

Run with:

```bash
docker compose -f docker/docker-compose.yml run --rm kani --proof verify_c1_division_remainder
```

The harnesses check fixed-point arithmetic. They do **not** check an entropy
statement.

## References

- [`../../../proofs/lean/src/KelvinProofs/FixedPoint.lean`](../../../proofs/lean/src/KelvinProofs/FixedPoint.lean)
- [`../../../proofs/lean/src/KelvinProofs/Integrator.lean`](../../../proofs/lean/src/KelvinProofs/Integrator.lean)
- `kelvin-core/src/fixed_math.rs` (`#[cfg(kani)]` module)
- [`../PROOF_INDEX.md`](../PROOF_INDEX.md)

## See also

- [PROOF_INDEX.md](../PROOF_INDEX.md)
- [C1 gap documents](.)
- [Lean proofs](../../../proofs/lean/README.md)
