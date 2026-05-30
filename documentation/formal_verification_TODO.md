# Formal Verification — Remaining Proof Gaps

> **Target Audience:** Math-specialized AI / PhD-level applied mathematician or cryptographer
> **Purpose:** Formally prove the quantum-resistance properties of the Kelvin chaos KDF
> **System Version:** Q32.64 fixed-point with Verlet/Euler integrators, SHAKE256 extraction
> **Status:** Draft for specialist review

---

## 1. Executive Problem Statement

The Kelvin cryptosystem derives cryptographic keystream from a deterministic fixed-point n-body gravitational simulation followed by SHAKE256 extraction. The security rests on the claim that this pipeline is a **quantum-resistant one-way function**: given the final keystream, it is computationally infeasible for any adversary (classical or quantum) to recover the initial orbital configuration or predict future keystream output.

**This document formalizes the remaining mathematical gaps** whose proof would certify the quantum-resistance claim. The target is rigorous mathematical proof sketches with explicit bounds suitable for future encoding in Coq, Lean, or integration with the existing Kani verification harnesses.

---

## 2. Complete System Definition

### 2.1 Q32.64 Fixed-Point Arithmetic

All simulation arithmetic uses Q32.64 fixed-point representation on `i128`:

- **Representation:** A fixed-point number `x` is stored as `x_raw = round(x · 2^64)`.
- **One unit:** `1.0 = 2^64` raw.
- **Addition/Subtraction:** Exact on `i128` — `(a ± b)_raw = a_raw ± b_raw`.
- **Multiplication:** `(a × b)_raw = (a_raw × b_raw) >> 64` using high/low splitting. Exact for all bounded inputs.
- **Division:** 192-iteration restoring division. Result satisfies `result × den ≈ num` with error bounded by `|den_raw| >> 64 + 2` ULPs.
- **Square root:** Binary digit-by-digit algorithm. Satisfies `result² ≈ val` with error bounded by `(2 × result_raw) >> 64 + 3` ULPs.
- **Physical domain:** Positions ∈ [−100 AU, 100 AU], masses ∈ (0, 1] M☉, velocities ∈ [−100, 100] AU/yr, where `1 AU = 2^64` raw.

### 2.2 Gravitational Simulation Loop

Given `N ≥ 3` bodies with masses `m_i`, positions `r_i`, velocities `v_i`, the simulation advances by discrete steps of size `dt = 2^54` raw (≈ 0.0156 yr).

**Acceleration computation** (performed once or twice per step):
```
For i ≠ j:
    r_ij = r_j − r_i
    dist_sq = |r_ij|² + ε²        (ε = 2^44 raw, softening)
    dist = sqrt(dist_sq)
    dist_cubed = dist_sq × dist
    a_i += G × m_j × r_ij / dist_cubed
```
where `G = 0x277A79937C8BBC0000` raw ≈ 39.478 AU³/(M☉·yr²).

**Verlet integrator** (symplectic, default):
```
1. v ← v + a × dt/2         (half kick)
2. r ← r + v × dt           (drift)
3. Compute new accelerations a'
4. v ← v + a' × dt/2        (half kick)
```

**Euler integrator** (non-symplectic, opt-in):
```
1. Compute accelerations a  (from current positions)
2. r ← r + v_old × dt      (position using OLD velocity)
3. v ← v + a × dt          (velocity update)
```

### 2.3 Extraction Pipeline

After `S` simulation steps, the final orbital state undergoes SHAKE256 extraction:

```
For each body i:
    hash(domain_sep || G_raw || ε_raw || S || N || i || m_i || r_i || v_i || a_i)

Output: 2048-byte entropy pool → keystream via SHAKE256 XOF
```

### 2.4 Default Configuration

| Parameter | Value | Notes |
|-----------|-------|-------|
| `N` | 5 | Number of bodies |
| `S` | 1,000,000 | Default simulation steps |
| `dt` | 2^54 | ≈ 0.0156 yr |
| `ε` | 2^44 | Softening factor |
| `min_separation` | 2^32 | Collision threshold (raw) |
| `ejection_threshold` | 0.5 | Specific energy threshold |
| `lyapunov_time` | ≈ 1443 steps | For standard 5-body config |

---

## 3. Conjecture Status Overview

| Conjecture | Code Infrastructure | Mathematical Proof | What's Missing |
|------------|-------------------|-------------------|----------------|
| **C1**: Information Loss | ✅ 3 Kani harnesses + empirical | ⚠️ 6 gaps | Uniformity, ε-bound, independence |
| **C2**: Lyapunov Certification | ✅ 3 Kani harnesses + empirical | ⚠️ 6 gaps | C·ε·S bound, Q32.64 QR, Kaplan-Yorke formalization |
| **C3**: Quantum Hardness | ✅ 2 Kani harnesses + empirical | ⚠️ 3 gaps (open research) | Quantum query lower bound |
| **C4**: Keystream Indistinguishability | ✅ 2 Kani harnesses + empirical | ⚠️ 3 gaps | Game-based reduction proof |
| **C5**: Configuration Space | ✅ 3 Kani harnesses + Monte Carlo | ⚠️ 4 gaps | Exact cardinality bound, min-entropy |

---

### C1: Fixed-Point Information Loss (Irreversibility) ✅

> **Status:** Proof sketch + Kani harnesses + empirical validation complete. See [`formal_verification.md`](formal_verification.md) (Section L1'). Results: `proofs/kani/results/c1_validation.log`.

**Kani-verified:**
1. `verify_c1_preimage_bound` — Local preimage ≤ 9 at softening limit
2. `verify_c1_epsilon_bound` — Global preimage ≤ 8,000,000 worst-case
3. `verify_c1_division_remainder` — Division remainder 0 ≤ r < den

**Remaining gaps (6 — 1 resolved):**

| # | Gap | Type | How to Fill |
|---|-----|------|------------|
| 1 | `k_op ≥ 1` bit derivation from Shannon entropy | Analytical | ✅ **RESOLVED** — see [`formal_verification_resolved/C1/gap1_kop_shannon_bound.md`](formal_verification_resolved/C1/gap1_kop_shannon_bound.md) |
| 2 | Uniform distribution of `dist_cubed` over physical range | Proof | Chaotic mixing + ergodicity → approximately uniform |
| 3 | Cumulative loss `S × k_step` saturation bound | Bound | Monotonic contraction of preimage set under repeated Φ |
| 4 | ε-bound `k_op ≥ 1 − ε` from first principles | Derivation | Express ε in terms of `log₂(max_dist_cubed / min_dist_cubed)` |
| 5 | Rounding error independence across pairs (Lemma A3) | Proof | Per-pair division/sqrt errors independent under chaotic mixing |
| 6 | Hartley vs Shannon entropy relationship for Φ | Formalization | Min-entropy per step ≥ Hartley entropy loss |

---

### C2: Finite-Precision Lyapunov Exponent Certification ✅

> **Status:** Proof sketch + Kani harnesses + empirical validation complete. See [`formal_verification.md`](formal_verification.md) (Section L2'). Results: `proofs/kani/results/c2_validation.log`.

**Kani-verified:**
1. `verify_pade_ln_bound` — Padé approximation monotonic and non-negative
2. `verify_lyapunov_division` — λ = ln_ratio / time finite and non-negative
3. `verify_perturbation_linear_regime` — δ = 2^40 raw produces O(δ) divergence

**Remaining gaps (6):**

| # | Gap | Type | How to Fill |
|---|-----|------|------------|
| 1 | `|λ_disc − λ_cont| ≤ C·ε·S` | Core bound | Error propagation through shadow orbit method; Lipschitz bounds on Verlet map |
| 2 | Positive λ lower bound `λ ≥ λ_min > 0` | Certification | Rigorous interval arithmetic on Benettin algorithm |
| 3 | Full Lyapunov spectrum in Q32.64 | Implementation | Fixed-point QR with proven rounding bounds; Wedin's theorem for deviation |
| 4 | Kaplan-Yorke as discrete-state entropy bound | Formalization | Translation from continuous D_KY to discrete-state bits in Q32.64 |
| 5 | Shadow orbit bias constant `C_bias` | Computation | Derived from variance across the 3 shadow axes |
| 6 | Lipschitz constant of λ w.r.t. parameters | Bounds | Concrete bounds for Padé ln and Q32.64 division in λ estimation |

---

### C3: Sequential Simulation Hardness Against Quantum Adversaries ✅

> **Status:** Proof sketch + Kani harnesses + empirical validation complete. See [`formal_verification.md`](formal_verification.md) (Section L3'). Results: `proofs/kani/results/c3_validation.log`.

**Kani-verified:**
1. `verify_c3_step_non_injective` — 1 ULP difference ≤ 10 ULPs after 1 step
2. `verify_c3_two_step_preimage_growth` — Preimage convergence after 2 steps

**Remaining gaps (3 — all open research):**

| # | Gap | Type | How to Fill |
|---|-----|------|------------|
| 1 | Extend Ambainis' adversary method to sequential dissipative Φ | Quantum complexity | New result: adversary lower bound for finite-state functions with per-step information loss |
| 2 | `Ω(2^{S·k/2})` query lower bound for Φ^S inversion | Theorem | Formal reduction: dissipative information loss → oracle query lower bound |
| 3 | Relate k to C1's ε-bound | Link | Quantify per-step Shannon loss in terms of C1's proven preimage bounds |

---

### C4: Computational Indistinguishability of the Keystream ✅

> **Status:** Proof sketch + Kani harnesses + empirical validation complete. See [`formal_verification.md`](formal_verification.md) (Section L4'). Results: `proofs/kani/results/c4_validation.log`.

**Kani-verified:**
1. `verify_extraction_deterministic` — Same state → same SHAKE256 output
2. `verify_domain_separation_functional` — Different tags → different outputs

**Remaining gaps (3):**

| # | Gap | Type | How to Fill |
|---|-----|------|------------|
| 1 | Game-based reduction proof | Cryptography | Distinguisher 𝒜 ⇒ SHAKE256 preimage finder OR chaos inverter via hybrid argument |
| 2 | Domain separation across all 6 modes | Extension | Covers Chaos, Photon, Quantum, Prism, Split, Flare (only 2 verified) |
| 3 | Advantage bound derivation | Formalization | `Adv(A) ≤ negl(n) + 2^{−S·k/2}` from SHAKE256 indifferentiability + chaos hardness |

---

### C5: Valid Configuration Space Cardinality ✅

> **Status:** Proof sketch + Kani harnesses + Monte Carlo validation complete. See [`formal_verification.md`](formal_verification.md) (Section C5). Results: `proofs/kani/results/c5_validation.log`.

**Kani-verified:**
1. `verify_c5_non_positive_mass` — Mass ≤ 0 rejected
2. `verify_c5_identical_positions` — Two bodies at same position rejected
3. `verify_c5_too_few_bodies` — Empty configuration rejected

**Remaining gaps (4):**

| # | Gap | Type | How to Fill |
|---|-----|------|------------|
| 1 | Exact cardinality lower bound `|Θ_5| ≥ 2^{1920}` | Combinatorics | Inclusion-exclusion for collision constraint; Liouville measure for bound orbit |
| 2 | Min-entropy `H_min ≥ 1800` bits | Proof | Show uniform distribution over Θ_5 under natural product measure |
| 3 | Stability reduction factor ≤ 2^{-100} | Bound | Phase-space volume satisfying specific energy < ejection threshold |
| 4 | Upgrade Kani harnesses to symbolic verification | Formal | Symbolic model checking of constraint rejection for all physically-bounded inputs |

---

## 4. Existing Proof Landscape (Already Verified in Kani)

These are the L0–L4 foundations. All code-level properties are verified; only the 23 mathematical gaps above remain.

| Level | What Is Proved | Location |
|-------|---------------|----------|
| **L0: Safety** | No panics, no overflows for bounded inputs | `kelvin-core/src/fixed_math.rs` |
| **L1: Functional Eq.** | Fixed ops match mathematical spec within error bounds | `proofs/kani/fixed_equivalence.rs` |
| **L2: Composite** | `compute_accelerations` satisfies Newton's laws | `proofs/kani/acceleration_proofs.rs` |
| **L3: Pipeline** | Verlet loop correctness, domain separation | `proofs/kani/pipeline_proofs.rs` |
| **L4: Determinism** | Bit-identical across platforms (SSE2/AVX/AVX2) | `tests/kelvin_tests/determinism.rs` |
| **L1': Information Loss** | Preimage bounds, division remainder, ε-bound | `kelvin-core/src/fixed_math.rs` |
| **L2': Lyapunov** | Padé ln, division safety, perturbation linear regime | `kelvin-kdf/src/lyapunov.rs` |
| **L3': Quantum** | Non-injectivity (1-ULP), preimage growth (2 steps) | `kelvin-core/src/fixed_math.rs` |
| **L4': Keystream** | Deterministic extraction, domain separation | `kelvin-kdf/src/extractor.rs` |
| **C5: Config** | Mass > 0, position uniqueness, minimum bodies | `kelvin-kdf/src/config.rs` |

---

## 5. References

- Apple Security Research (2026). "Formal verification of corecrypto for post-quantum cryptography." security.apple.com/blog/formal-verification-corecrypto/
- Kani Rust Verifier. https://model-checking.github.io/kani/
- Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980). "Lyapunov Characteristic Exponents." *Meccanica*, 15, 9–20.
- Poincaré, H. (1899). *Les Méthodes Nouvelles de la Mécanique Céleste*, Vol. 3.
- Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration* (2nd ed.). Springer.
- Shannon, C. E. (1949). "Communication Theory of Secrecy Systems." *Bell System Technical Journal*, 28(4), 656–715.
- Bennett, C. H., et al. (1997). "Strengths and Weaknesses of Quantum Computing." *SIAM J. Comput.*, 26(5), 1510–1523.
- Ambainis, A. (2002). "Quantum Lower Bounds by Quantum Arguments." *J. Comput. Syst. Sci.*, 64(4), 750–767.
- National Institute of Standards and Technology. (2015). "SHA-3 Standard." FIPS PUB 202.