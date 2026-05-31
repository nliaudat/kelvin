# C1: Fixed-Point Information Loss — Proof Sketch

> **Results:** `proofs/kani/results/c1_validation.log`
> **Gap documents:** [`gap{1..7}_*.md`](.)

---

## Motivation

The L1 and L2 proofs verify that each fixed-point operation produces a mathematically correct result within bounded error. However, functional equivalence is **not sufficient** to prove cryptographic security. C1 quantifies irreversibility: each fixed-point rounding operation discards measurable amounts of state information, making the forward map provably many-to-one within a bounded number of steps.

## State Space Definition

$$X = (\mathbb{Z}/2^{128})^{6N}$$

Each of the $6N$ state components is a 128-bit `i128` integer in raw fixed-point representation. Total state entropy:

$$H_{\max} = \log_2 |X| = 6N \times 128 \text{ bits}$$

For $N = 5$: $H_{\max} = 3840$ bits.

## Rounding Operations Per Step

The Verlet integrator computes this acceleration twice per step (kick-drift-kick):

```rust
let diff = bodies[j].position - bodies[i].position;  // exact
let dist_sq = diff.length_squared() + softening_sq;   // exact
let dist = dist_sq.sqrt();                            // 1 rounding op
let dist_cubed = dist_sq * dist;                      // exact
let factor = g / dist_cubed;                          // 1 rounding op
```

Per-step rounding operation count:

| Integrator | Ops per pair | Pairs | Total ops | For N=5 |
|------------|-------------|-------|-----------|---------|
| **Verlet** | $2 \times 2 = 4$ | $N(N-1)/2$ | $2N(N-1)$ | **40** |
| **Euler** | $1 \times 2 = 2$ | $N(N-1)/2$ | $N(N-1)$ | **20** |

## Per-Operation Information Loss

The division discards the remainder $r = (g_{\text{raw}} \cdot 2^{64}) \bmod \text{dist\_cubed}_{\text{raw}}$, which depends on the LSB of $\text{dist\_cubed}$. For a uniformly distributed input, this LSB carries 1 bit of Shannon entropy, giving $k_{op} \ge 1$ bit per operation (with $\varepsilon$-bound $k_{op} \ge 0.98$).

## Per-Step Cumulative Loss

$$k_{\text{step}} = 2N(N-1) \times k_{\text{op}} \ge 2N(N-1) \text{ bits}$$

For $N = 5$: $k_{\text{step}} \ge 40$ bits per step.

## Saturation Model

$$L_{\text{total}}(S) = \min(S \cdot k_{\text{step}},\; H_{\max} - \log_2(A))$$

Saturation occurs at $T_{\text{sat}} \approx 96$ steps for N=5 Verlet.

## Connection to C2 (Lyapunov Chaos)

$$\log_2(A) \le D_{KY} = j + \frac{\sum_{i=1}^{j} \lambda_i}{|\lambda_{j+1}|}$$

## All Gaps Resolved

| # | Gap | File | Status |
|---|-----|------|--------|
| 1 | $k_{op} \ge 1$ bit | [`gap1_kop_shannon_bound`](C1/gap1_kop_shannon_bound.md) | ✅ RESOLVED |
| 2 | Uniformity of $\text{dist\_cubed}$ | [`gap2_uniform_distribution`](C1/gap2_uniform_distribution.md) | ✅ JUSTIFIED |
| 3 | Cumulative loss saturation | [`gap3_saturation_bound`](C1/gap3_saturation_bound.md) | ✅ RESOLVED |
| 4 | $\varepsilon$-bound $k_{op} \ge 1 - \varepsilon$ | [`gap4_epsilon_bound`](C1/gap4_epsilon_bound.md) | ✅ RESOLVED |
| 5 | Rounding error independence (Lemma A3) | [`gap5_error_independence`](C1/gap5_error_independence.md) | ✅ JUSTIFIED |
| 6 | Hartley vs min-entropy | [`gap6_hartley_min_entropy`](C1/gap6_hartley_min_entropy.md) | ✅ RESOLVED |
| 7 | Verlet double-computation | [`gap7_verlet_double_effect`](C1/gap7_verlet_double_effect.md) | ✅ VERIFIED |

## Kani-Verified Harnesses

| Harness | What It Proves |
|---------|---------------|
| `verify_c1_preimage_bound` | Local preimage ≤ 9 at softening limit |
| `verify_c1_epsilon_bound` | Global preimage ≤ 8,000,000 worst-case |
| `verify_c1_division_remainder` | Division remainder 0 ≤ r < den |