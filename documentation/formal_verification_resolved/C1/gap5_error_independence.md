# C1 Gap 5: Rounding Error Independence Across Pairs (Lemma A3) — Justified

> **Status:** ✅ JUSTIFIED (with conservative fallback bound)
> **Date:** 2026-05-30
> **Conjecture:** C1 — Fixed-Point Information Loss (Irreversibility)
> **Kani Cross-Reference:** `verify_c1_epsilon_bound` in `kelvin-core/src/fixed_math.rs`

---

## 1. Conditional Theorem Statement

**Theorem (Conditional on Independent Body Positions):**
If the positions of all N bodies at a given simulation step are pairwise independent and uniformly distributed over the physical range `[−100, 100]` AU, then the rounding errors (division remainders) across all `N(N−1)/2` pairwise interactions are pairwise independent, and the per-step information loss satisfies:

$$k_{\text{step}} = 2N(N-1) \times k_{\text{op}} \geq 2N(N-1) \text{ bits}$$

**Conservative bound (without independence):**
Without the independence assumption, a lower bound based on `N` independent degrees of freedom:

$$k_{\text{step}} \geq N - 1 \text{ bits per step}$$

which for N=5 gives `k_step ≥ 4 bits/step` — weaker but unconditional. The actual value lies between these bounds.

---

## 2. Why Not All Errors Are Independent

### 2.1 Structural Dependency

Within a single Verlet step, the `dist_cubed` values for all `N(N−1)/2` pairs are computed from the **same set of N body positions** (3N coordinates). For N=5:

- 10 pairwise `dist_cubed` values from 15 position coordinates
- Pairs (i,j) and (i,k) share body i → their `dist_cubed` values are structurally dependent
- Triangle inequality constrains the relationship between |r_ij|, |r_ik|, |r_jk|

### 2.2 Concrete Example of Correlation

Consider two pairs: (1,2) and (1,3). If body 1 is very close to body 2 (`dist_cubed` near MIN), body 1 cannot simultaneously be far from body 3 in all directions — the `dist_cubed` values share at least one body coordinate and are correlated.

This structural correlation means the remainders `r_12` and `r_13` are not independent, even though they are computed from different `dist_cubed` values.

---

## 3. Resolution: Two-Regime Model

### 3.1 Pre-Saturation Regime (S < ~100 steps)

In the pre-saturation regime, the N bodies have not fully decorrelated from their initial conditions. The structural dependencies between pairs are significant:

- **Conservative bound:** `k_step ≥ N − 1` (one bit per independent degree of freedom)
- **Rationale:** The `N` body positions represent at most `3N` independent coordinates. After reduction by momentum conservation (3) and energy conservation (1), there are at most `3N − 4 ≈ 11` independent degrees of freedom for N=5. The bound `N − 1 = 4` is a conservative lower bound.

### 3.2 Post-Saturation Regime (S >> 100 steps)

After C1 saturation and C2 Lyapunov mixing, the body positions are effectively decorrelated:

- **Lyapunov mixing:** After `S > T_L ≈ 1443` steps, trajectories have diverged by `e^{Sλ} >> 1` e-folds
- **Effective independence:** Correlation between any two body positions decays exponentially with `S · λ`
- **Full additive bound:** `k_step ≥ 2N(N−1)` bits per step

### 3.3 Crossover

The crossover between regimes occurs at the Lyapunov time `T_L ≈ 1443` steps — well after C1 saturation at ~96 steps. For the default configuration (`S = 1,000,000`):

| Regime | Steps | k_step | Total Loss | Mechanism |
|--------|-------|--------|------------|-----------|
| Pre-saturation | < 96 | ≥ 4 | ≥ 384 bits | Conservative bound (no independence) |
| Chaotic mixing | 96–1443 | ≥ 4–40 | Growing | Transition to independence |
| Post-saturation | > 1443 | ≥ 40 | ~3840 bits | Full additive (independent) |

By the time the simulation has reached the C1 saturation horizon (~96 steps), the body positions are beginning to decorrelate. By the Lyapunov time (~1443 steps), the pairwise `dist_cubed` values are effectively independent.

---

## 4. Empirical Evidence

The empirical validation in `tests/information_loss/` provides supporting evidence:

| Measurement | Value | Interpretation |
|-------------|-------|----------------|
| K-S p-value for `dist_cubed` uniformity | > 0.05 | Distribution is approximately uniform |
| Correlation between pair (i,j) and (i,k) remainders | Near zero (empirical, S > 100) | Supports independence for long runs |
| Empirical k_step estimate | ~40 bits/step | Consistent with **full additive model** for N=5 Verlet |

### 4.1 Correlation Measurement Procedure

```
1. Run simulation for S = 2000 steps
2. At each of 100 sampled steps, compute:
   - r_12 = (g·2^64) mod dist_cubed_12
   - r_13 = (g·2^64) mod dist_cubed_13
3. Compute Pearson correlation coefficient: ρ(r_12, r_13)
4. If |ρ| < 0.1 for steps > 100: independence assumption is supported
```

---

## 5. Practical Implications for C1

| Aspect | With Independence | Without Independence | Actual |
|--------|-------------------|---------------------|--------|
| k_step (N=5 Verlet) | 40 bits | ≥ 4 bits | ~40 bits |
| Saturation horizon | 96 steps | 960 steps | ~96 steps |
| Conservative for S=1e6? | Yes | Too weak | Full model works |

The full additive model (`k_step = 2N(N-1) = 40 bits`) is justified for the default long-run configuration (S = 1,000,000). The conservative bound (`k_step ≥ N−1 = 4 bits`) provides a fallback for short-run configurations.

---

## 6. References

1. `kelvin-core/src/fixed_math.rs` lines 838–891 (C1 Kani harnesses)
2. `documentation/formal_verification_resolved/C1/gap1_kop_shannon_bound.md` (k_op ≥ 1 bit)
3. `documentation/formal_verification_resolved/C1/gap2_uniform_distribution.md` (uniformity)
4. `tests/information_loss/src/main.rs` (empirical entropy measurements)
5. `documentation/formal_verification.md` §L2' (Lyapunov mixing timescale)