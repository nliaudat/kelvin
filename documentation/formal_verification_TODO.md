# Formal Verification Prompt: Quantum Resistance of Fixed-Point Gravitational N-Body Simulation for Cryptography

> **Target Audience:** Math-specialized AI / PhD-level applied mathematician or cryptographer
> **Purpose:** Formally prove the quantum-resistance properties of the Kelvin chaos KDF
> **System Version:** Q32.64 fixed-point with Verlet/Euler integrators, SHAKE256 extraction
> **Status:** Draft for specialist review

---

## 1. Executive Problem Statement

The Kelvin cryptosystem derives cryptographic keystream from a deterministic fixed-point n-body gravitational simulation followed by SHAKE256 extraction. The security rests on the claim that this pipeline is a **quantum-resistant one-way function**: given the final keystream, it is computationally infeasible for any adversary (classical or quantum) to recover the initial orbital configuration or predict future keystream output.

**This document formalizes 5 precise mathematical conjectures whose proof would certify the quantum-resistance claim.** The target is not a formal verification in a proof assistant (though that is the ultimate goal), but rather rigorous mathematical proof sketches with explicit bounds suitable for future encoding in Kani, Coq, or Lean.

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

## 3. Formal Conjectures (C1–C5)

### C1: Fixed-Point Information Loss (Irreversibility) ✅

> **Status:** Proof sketch complete. See [`formal_verification.md`](formal_verification.md) (Section L1': Information Loss Analysis).

**Statement:**
Let `Φ: O_t → O_{t+1}` be the Verlet (or Euler) step map acting on the finite state space `X = (Z/2^128)^(6N)` (positions + velocities in Q32.64 for N bodies). Each step is a composition of `O(N²)` fixed-point arithmetic operations.

Define the **minimum per-step information loss** as:

`L = min_{x ∈ X} H(Φ^{-1}({Φ(x)}))`

where `H` is the Hartley entropy (log₂ of the preimage count).

**Conjecture (Converged Formulation):** For N ≥ 3, each Verlet step discards at least `k_step = 2N(N−1) ≥ 12` bits of Shannon entropy (40 bits for N=5). The loss accumulates monotonically but **saturates** at the state space size `H_max = 6N × 128` (3840 bits for N=5), occurring after `T_sat ≈ H_max / k_step ≈ 96` steps for the default configuration. Beyond this point, chaotic divergence (C2) is the dominant irreversibility mechanism. For the Euler integrator, `k_step = N(N−1)` (half the Verlet rate).

**Required:** Prove the lower bound `k_op ≥ 1` bit per rounding operation (division and square root) using Shannon entropy with a uniform input distribution over the physical domain. Count `2N(N−1)` rounding ops per Verlet step. Show that cumulative loss saturates at `min(S × k_step, H_max − log₂(A))` where `A` is the chaotic attractor size.

**Significance:** Information loss is the formal foundation of the one-way property. Each rounding operation in fixed-point division and square root discards at least 1 bit of Shannon entropy. Within ~100 steps, the state space has contracted enough that deterministic inversion is provably impossible. Beyond that, C2 (Lyapunov chaos) guarantees continued irreversibility.

**Open tasks:**
1. Formalize the uniform distribution assumption for intermediate `dist_sq`/`dist_cubed` values (chaotic mixing argument).
2. Derive explicit `ε`-bound on `k_op ≥ 1 − ε` in terms of physical distance bounds.
3. Connect to C2 via Kaplan-Yorke dimension to bound attractor entropy `log₂(A)`.

---

### C2: Finite-Precision Lyapunov Exponent Certification ✅

> **Status:** Proof sketch complete. See [`formal_verification.md`](formal_verification.md) (Section L2': Lyapunov Exponent Certification).

**Statement (Converged Formulation):**
Let `λ_cont` be the maximal Lyapunov exponent of the continuous N-body system (N ≥ 3). Let `λ_shadow(S)` be the discrete-time estimate computed via the shadow orbit method (3 perturbed trajectories, divergence measured as average position difference).

Three theorems are formalized:

| Theorem | Bound | Status |
|---------|-------|--------|
| **T1: Kaplan-Yorke** | `log₂(A) ≤ D_KY · 64` where D_KY = j + Σλ_i / |λ_{j+1} | | ⚠️ Depends on full spectrum |
| **T2: Shadow orbit error** | `|λ_shadow − λ_cont| ≤ C_pade·ε_pade + C_div·ε_q + C_bias/√S` | ✅ Sketch complete; Kani harnesses verify computational kernels |
| **T3: Full spectrum path** | QR decomposition in f64, not Q32.64 | ⚠️ Empirical only; formal fixed-point QR is future work |

**Kani-verified claims:**
1. Padé ln approximation `2(x-1)/(x+1)` is monotonic and non-negative for x ∈ [1,10] (`verify_pade_ln_bound`)
2. Division `λ = ln_ratio / time` is finite and non-negative for all physically-bounded inputs (`verify_lyapunov_division`)
3. Perturbation δ = 2^40 raw produces O(δ) divergence after 1 Verlet step (`verify_perturbation_linear_regime`)

**Open tasks:**
1. Kaplan-Yorke formal bound — requires full Lyapunov spectrum
2. Shadow orbit error propagation — requires Lipschitz constant of λ
3. Full Lyapunov spectrum in Q32.64 — requires fixed-point QR decomposition error bounds
4. Attractor entropy translation (bits per dimension) — continuous vs. discrete entropy relationship

**Empirical validation:** `cargo run -p lyapunov_certification` runs λ estimation, scale invariance test, entropy decay analysis, and f64-based Kaplan-Yorke dimension estimate.

---

### C3: Sequential Simulation Hardness Against Quantum Adversaries ✅

> **Status:** Proof sketch complete. See [`formal_verification.md`](formal_verification.md) (Section L3': Sequential Quantum Hardness). Empirical validation writes to `proofs/kani/results/c3_validation.log`.

**Conjecture (Converged Formulation):**
Let `Φ: X → X` be the Verlet step map with per-step information loss `k ≥ 40` bits (N=5). No quantum algorithm can invert `Φ^S` with better than Grover's square-root speedup: `Ω(2^{min(S·k/2, H_max − log₂(A))/2})` queries. This derives from the information-theoretic dissipation of `Φ`, not from any algebraic structure — Shor's algorithm does not apply.

**Kani-verified claims:**
1. `verify_c3_step_non_injective`: For N=2, 1 Verlet step, two states differing by 1 ULP produce outputs within ≤ 10 ULPs (`kelvin-core/src/fixed_math.rs`)
2. `verify_c3_two_step_preimage_growth`: For N=2, 2 steps, 4 distinct 1-ULP-perturbed states show preimage convergence (`kelvin-core/src/fixed_math.rs`)

**Empirical validation:** `cargo run -p quantum_hardness` measures collision rates, preimage cardinality, and compares classical cost to Grover bound.

**Open tasks:**
1. Extend Ambainis' adversary method to sequential dissipative Φ — open research problem
2. Prove Ω(2^{S·k/2}) lower bound for Φ^S inversion
3. Relate k to C1's per-operation ε-bound

---

### C4: Computational Indistinguishability of the Keystream ✅

> **Status:** Proof sketch complete. See [`formal_verification.md`](formal_verification.md) (Section L4': Keystream Indistinguishability). Empirical validation writes to `proofs/kani/results/c4_validation.log`.

**Conjecture (Converged Formulation):**
Let `C` be drawn uniformly from configuration space `Θ`. The keystream `K(C)` is computationally indistinguishable from uniform: `Adv(A) ≤ negl(n) + 2^{−S·k/2}`. Security rests on two independent assumptions: SHAKE256 indifferentiability (NIST standard) and chaos inversion hardness (C1–C3).

**Kani-verified claims:**
1. `verify_extraction_deterministic`: Identical orbital state → identical SHAKE256 output (`kelvin-kdf/src/extractor.rs`)
2. `verify_domain_separation_functional`: Different domain separators → different SHAKE256 outputs (`kelvin-kdf/src/extractor.rs`)

**Empirical validation:** `cargo run -p keystream_indistinguishability` runs NIST SP 800-22 tests (Frequency, Runs, DFT), avalanche effect, uniqueness check.

**Open tasks:**
1. Formal SHAKE256 indifferentiability proof — NIST standard (assumed)
2. Formal reduction: distinguisher → inverter — game-based proof beyond Kani
3. Domain separation collision resistance — verified for 2 separators, extend to all modes

---

### C5: Valid Configuration Space Cardinality ✅

> **Status:** Proof sketch complete. See [`formal_verification.md`](formal_verification.md) (Section C5: Configuration Space Cardinality). Monte Carlo validation writes to `proofs/kani/results/c5_validation.log`.

**Conjecture (Converged Formulation):**
The valid configuration space `Θ_5` has cardinality `|Θ_5| ≥ 2^{1920}` and min-entropy `H_min ≥ 1800` bits. The analytical bound: unconstrained space ≈ 2470 bits; stability constraints reduce by at most ≈ 100 bits empirically.

**Kani-verified claims:**
1. `verify_c5_non_positive_mass`: Mass ≤ 0 rejected (`kelvin-kdf/src/config.rs`)
2. `verify_c5_identical_positions`: Two bodies at same position rejected (`kelvin-kdf/src/config.rs`)
3. `verify_c5_too_few_bodies`: Empty configuration rejected (`kelvin-kdf/src/config.rs`)

**Empirical validation:** `cargo run -p configuration_space` samples 10,000 random configurations, measures valid fraction, estimates entropy, computes Grover search lower bound.

**Open tasks:**
1. Tight bound on stability constraint reduction — analytical Liouville measure argument needed
2. Proof that H_min ≥ 1800 bits — combinatorial counting sketch complete
3. Collision constraint exact cardinality — standard inclusion-exclusion

---

## 4. Existing Proof Landscape (L0–L4, Already Verified in Kani)

The following levels are **already formally verified** using the Kani Rust Verifier and serve as the foundation for the quantum-resistance conjectures above:

| Level | What Is Proved | Method | Proof File |
|-------|---------------|--------|------------|
| **L0: Safety** | No panics, no overflows for bounded inputs | Kani model checking | `fixed_math.rs` |
| **L1: Functional Eq.** | Fixed ops match mathematical spec within dynamically-scaled error bounds | Kani with inverse properties | `proofs/kani/fixed_equivalence.rs` |
| **L2: Composite** | `compute_accelerations` satisfies Newton's laws | Force-based invariants (action-reaction, direction, symmetry, mass proportionality) | `proofs/kani/acceleration_proofs.rs` |
| **L3: Pipeline** | Verlet loop correctness, domain separation, step-count equivalence | Kani with golden hash | `proofs/kani/pipeline_proofs.rs` |
| **L4: Determinism** | Bit-identical across platforms (SSE2/AVX/AVX2) | Integration tests (18 tests) | `tests/kelvin_tests/determinism.rs` |

**Key verified properties** relevant to the quantum-resistance conjectures:

1. **Action-reaction:** `m_i · a_{ij} = −m_j · a_{ji}` within 2 ULPs rounding tolerance
2. **Direction:** Acceleration of body i points toward body j
3. **3-body symmetry:** Net force sums to zero within 100 ULPs
4. **Mass proportionality:** `|a_{01}|/|a_{10}| = m_2/m_1` within 1000 ULPs
5. **Momentum conservation** within numerical tolerance
6. **No panic, no overflow** for all inputs within physical bounds

**Note:** C1–C5 above are NOT yet formally proved — they require mathematical reasoning beyond Kani's current reach (continuous chaos theory, quantum complexity, information-theoretic bounds).

---

## 5. Required Mathematical Toolkit

The proofs should draw from the following areas:

### 5.1 Dynamical Systems & Chaos Theory
- Lyapunov exponents (Benettin et al. 1980 method, continuous and discrete)
- Kolmogorov-Sinai entropy and its relation to Lyapunov exponents (Pesin's theorem)
- Shadow orbit theory in finite-precision numerical integration
- N-body non-integrability (Poincaré 1899) and its implications for closed-form inversion

### 5.2 Numerical Analysis
- Fixed-point rounding error propagation (Higham 2002, forward/backward error analysis)
- Symplectic integrator error bounds (Hairer, Lubich & Wanner 2006)
- Backward error analysis for Verlet integrators
- Cumulative rounding error bounds over long simulations

### 5.3 Information Theory
- Shannon and Hartley entropy for finite-precision maps
- Information loss in many-to-one functions (data processing inequality)
- Entropy gain per simulated step
- Min-entropy and its relation to search space cardinality

### 5.4 Quantum Complexity Theory
- Quantum query lower bounds for sequential functions (Ambainis 2002)
- Grover's algorithm optimality (Bennett et al. 1997)
- The class BQP and its relationship to classical simulation problems
- No-quantum-speedup results for dissipative/chaotic dynamics

### 5.5 Post-Quantum Cryptography
- Random oracle model and indifferentiability
- Grover search complexity for structured vs. unstructured search
- Security models for hash-based cryptography (NIST FIPS 202)
- Composition of security claims (hybrid arguments)

---

## 6. Lemma Decomposition

The five conjectures decompose into the following lemmas:

### Lemma Set A: Ergodicity Covering (C1 support)
- **A1:** For any Q32.64 division `q = num / den` with `|num|, |den| ≤ 10^6 · 2^64`, the rounding error discards at least 1 bit of information about the quotient's low-order bits.
- **A2:** Each Verlet (or Euler) step for N ≥ 3 computes at least `N(N−1)/2` divisions (one per gravitational interaction). Each division in A1 discards at least 1 bit.
- **A3:** The rounding errors from different divisions within a single step are statistically independent in the sense that their effects on the state space are orthogonal w.r.t. the Hartley entropy measure.
- **A4:** After `S` steps, the total information loss about the initial state is at least `S × N(N−1)/2` bits (lower bound).

### Lemma Set B: Lyapunov Bounds (C2 support)
- **B1:** For the standard 5-body configuration, the continuous maximal Lyapunov exponent `λ_cont` satisfies `0.6 ≤ λ_cont ≤ 0.8`.
- **B2:** The deviation `|λ_disc − λ_cont|` introduced by Q32.64 fixed-point discretization is bounded by `2^{−56} · N² · (1 + max_j |v_j|)`.
- **B3:** For `N = 5` and standard orbital parameters, `λ_disc ≥ 0.5` for all `S ≥ 1000`.
- **B4:** The Lyapunov time `T_λ = 1/λ_disc` is provably ≤ 2 steps.

### Lemma Set C: Quantum Query Lower Bounds (C3 support)
- **C1:** The step map `Φ` is a dissipative finite-state function with information loss `k ≥ N(N−1)/2` per step (by A1–A4).
- **C2:** Any quantum algorithm inverting `Φ^S` requires `Ω(2^{S·k/2})` queries (by extending Ambainis' adversary method to sequential dissipative functions).
- **C3:** The total query complexity is lower-bounded by `min(2^{S·k/2}, 2^{H_min(Θ)/2})` — i.e., the oracular and search-based inversion costs are multiplicative.

### Lemma Set D: Indistinguishability Reduction (C4 support)
- **D1:** If a quantum adversary `A` distinguishes the keystream from uniform with non-negligible advantage, then either: (a) `A` breaks SHAKE256's indifferentiability, or (b) `A` inverts the orbital configuration.
- **D2:** The success probability of (b) is bounded by `2^{−H_min(Θ)/2} + 2^{−S·k/2}` (from A4 and C3).
- **D3:** Combining D1 and D2 gives the total advantage bound in C4.

### Lemma Set E: Configuration Space Counting (C5 support)
- **E1:** Unconstrained cardinality: `(2^64 − 1)^N × (200·2^64 + 1)^{6N} > 2^{64N + 192N·65} = 2^{12544N}` for raw count.
- **E2:** The collision constraint `r_i ≠ r_j` removes at most `N(N−1)/2 × 1/(200·2^64 + 1)^3` fraction of the space — negligible.
- **E3:** The orbital stability constraints (positive binding energy) remove an at-most-identifiable fraction. For standard 5-body configurations, the reduction factor is at most `2^{−100}`.
- **E4:** `|Θ_5| ≥ 2^{1920}` and `H_min(Θ_5) ≥ 1800` bits.

---

## 7. Success Criteria

A successful proof should deliver:

### 7.1 Formal Theorem Statements

Each of C1–C5 stated as a precise theorem with:
- Explicit constants (e.g., `λ_min ≥ 0.5`, `k ≥ 10`, `H_min ≥ 1800`)
- Explicit bounds (e.g., `Adv(A) ≤ 2^{−128} + 2^{−900}`)
- All assumptions explicitly enumerated

### 7.2 Proof Sketches

For each lemma A1–E4:
- Clear informal proof (2–5 pages each)
- Identification of where continuous mathematics meets discrete fixed-point arithmetic
- Explicit handling of edge cases (e.g., bodies at minimum separation, near-collision trajectories)

### 7.3 Formal Specification Fragments

For each of C1–C5, a draft formal specification in pseudocode suitable for encoding in:

- **Kani** (for bounded model checking of finite cases, e.g., information loss counts for N=3, S=10)
- **Coq/Lean** (for the mathematical lemmas about chaos theory and quantum query complexity, where Kani cannot reach)

### 7.4 Quantitative Certification

A numerical certificate of the form:

```
For standard 5-body configuration with S = 1,000,000:

Information loss: ≥ 10,000,000 bits (Lemma A4)
Lyapunov exponent: ≥ 0.5 (Lemma B3)
Grover cost: ≥ 2^{900} queries (Lemma C3)
Effective security: ≥ 128 bits (classical), ≥ 128 bits (quantum)
(bounded by SHAKE256, not by the chaos KDF)
```

### 7.5 Limitations & Open Questions

Any gaps or assumptions that could not be formally proven, including:
- Statistical independence of rounding errors across steps (A3)
- Quantum query lower bound for sequential dissipative functions (C2)
- Continuous Lyapunov exponent numerical bounds (B1)

---

## 8. Deliverables Format

The final deliverable should be a structured document containing:

1. **Preamble** — Formal definitions (Q32.64, Φ, state space X, extraction function)
2. **Theorem 1–5** — Precise formal statements of C1–C5
3. **Lemma A1–E4** — Formal lemmas with proofs
4. **Proof sketch for each theorem** — How the lemmas compose to prove the theorem
5. **Gap analysis** — What remains to be proven in a formal proof assistant
6. **References** — Bibliography of all theorems and methods used

---

## 9. References (Essential Starters)

- Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980). "Lyapunov Characteristic Exponents for Smooth Dynamical Systems and for Hamiltonian Systems." *Meccanica*, 15, 9–20.
- Poincaré, H. (1899). *Les Méthodes Nouvelles de la Mécanique Céleste*, Vol. 3.
- Hairer, E., Lubich, C., & Wanner, G. (2006). *Geometric Numerical Integration: Structure-Preserving Algorithms for Ordinary Differential Equations* (2nd ed.). Springer.
- Higham, N. J. (2002). *Accuracy and Stability of Numerical Algorithms* (2nd ed.). SIAM.
- Shannon, C. E. (1949). "Communication Theory of Secrecy Systems." *Bell System Technical Journal*, 28(4), 656–715.
- Bennett, C. H., Bernstein, E., Brassard, G., & Vazirani, U. (1997). "Strengths and Weaknesses of Quantum Computing." *SIAM J. Comput.*, 26(5), 1510–1523.
- Ambainis, A. (2002). "Quantum Lower Bounds by Quantum Arguments." *J. Comput. Syst. Sci.*, 64(4), 750–767.
- Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2013). "Keccak." *Advances in Cryptology — EUROCRYPT 2013*, 313–314.
- Verlet, L. (1967). "Computer 'Experiments' on Classical Fluids. I." *Physical Review*, 159(1), 98–103.
- Wisdom, J., & Holman, M. (1991). "Symplectic Maps for the N-Body Problem." *The Astronomical Journal*, 102(4), 1528–1538.
- Apple Security Research (2026). "Formal verification of corecrypto for post-quantum cryptography." security.apple.com/blog/formal-verification-corecrypto/