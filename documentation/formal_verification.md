# Formal Verification — Kelvin Cryptosystem

> **Status:** ✅ **All 23 gaps closed (19 proved, 4 justified, 0 open).**
> - **Lean Proofs**: Mathematical specifications in `proofs/lean/src/` (7 files, 32 theorems)
> - **Kani Proofs**: Code verification in `proofs/kani/*.rs`
> - **Gap Documentation**: Detailed analysis in `formal_verification/C*/gap*.md`
> 
> The L0–L4 Kani proofs verify the code matches its specification. The C1–C5 conjectures are empirical arguments and plausibility estimates — they are **not** formal security reductions; C1 gap2/gap5, C2 gap3, and C5 gap3 are closed by argument + measurement, not proof. Worked numerics assume the 5-body analysis config (current standard is 8-body). See [`formal_verification/`](formal_verification/) for complete proof documents.
> **Last Updated:** 2026-11-21

## 1. Proof Architecture

```
Mathematical Specification (Lean) ← proofs/lean/src/
├── FixedPoint.lean              → Q32.64 arithmetic (L0, L1, C1)
├── NewtonianGravity.lean        → Gravitational forces (L2)
├── VerletIntegrator.lean        → Symplectic integration (L3)
├── LyapunovExponents.lean       → Chaos certification (C2)
├── Shake256Extraction.lean      → Entropy extraction (C4)
└── HKDFDerivation.lean          → Key schedule (L3, C4)
    ↓ equivalence (proven below)
Code Specification (Rust) ← kelvin-core/src/
├── fixed_math.rs                → Q32.64 arithmetic
├── body.rs                      → Vec3, OrbitalBody
└── integrator.rs                → Verlet, Euler
    ↓ equivalence (Kani verified)
Code Verification (Kani) ← proofs/kani/*.rs
└── Automated model checking and golden hash verification
```

### 1.1 Code-Level Verification

The L1 (Functional Equivalence) and L2 (Composite Correctness) proofs use the
**Apple-inspired** dynamic error bound strategy detailed in
[`formal_verification/code_verification.md`](formal_verification/code_verification.md).

---

## 2. Executive Summary

The Kelvin cryptosystem derives cryptographic keystream from a deterministic fixed-point n-body gravitational simulation followed by SHAKE256 extraction. The security rests on the claim that this pipeline is a **quantum-resistant one-way function** (an unproven conjecture — see [Security Assumptions](security_assumptions.md)): given the final keystream, it is computationally infeasible for any adversary (classical or quantum) to recover the initial orbital configuration or predict future keystream output.

**All 23 implementation correctness gaps (L0–L4) are resolved.** The Kani proofs verify the code matches its specification, but do NOT prove security. The C1–C5 conjectures are empirical arguments and plausibility estimates.

The conjecture chain is:

```
C1 (k_step ≥ 40 bits/step) → empirical information-loss measurement
  → C2 (λ > 0, Kaplan-Yorke attractor bound) → Lyapunov certification
  → C3 (Ω(2^960) Grover bound via Θ search) → quantum search estimate over theoretical config space
  → C5 (|Θ₅| ≥ 2^1920) → theoretical config cardinality estimate
  → C4 (Adv(A) ≤ negl(n) + 2^-960) → distinguishing advantage estimate (see caveat below)
```

> ⚠️ **Important caveat on C4**: The `2⁻⁹⁶⁰` term is derived from C3's Grover bound over the theoretical configuration space. This assumes unstructured quantum search over the full config space (2¹⁹²⁰). In practice the reachable keyspace is limited by the OS CSPRNG (2²⁵⁶), and the Grover oracle may have exploitable structure. The system's effective post-quantum security, bounded by SHAKE256's Grover resistance, is **128-bit**. See `stream_cipher_security.md` §1 for the effective bound.

---

## 3. Proof Levels

| Level | What Is Proved | Location |
|-------|---------------|----------|
| **L0: Safety** | No panics, no overflows for bounded inputs | [`proofs/lean/src/fixed_point/FixedPoint.lean`](proofs/lean/src/fixed_point/FixedPoint.lean) + Kani |
| **L1: Functional Eq.** | Fixed ops match mathematical spec within error bounds | [`proofs/lean/src/fixed_point/FixedPoint.lean`](proofs/lean/src/fixed_point/FixedPoint.lean) + Kani |
| **L2: Composite** | `compute_accelerations` satisfies Newton's laws | [`proofs/lean/src/n_body/NewtonianGravity.lean`](proofs/lean/src/n_body/NewtonianGravity.lean) + Kani |
| **L3: Pipeline** | Verlet loop correctness, domain separation | [`proofs/lean/src/dynamics/VerletIntegrator.lean`](proofs/lean/src/dynamics/VerletIntegrator.lean) + Kani |
| **L4: Determinism** | Bit-identical across platforms (SSE2/AVX/AVX2) | `tests/kelvin_tests/determinism.rs` |

**These are implementation correctness proofs — they do not prove cryptographic security.**

The following are empirical estimates and plausibility arguments (not formal proofs):

| Level | Description | Type |
|-------|-------------|------|
| **[C1: Information Loss](formal_verification/C1/readme.md)** | Per-step fixed-point rounding irreversibility | Empirical measurement |
| **[C2: Lyapunov](formal_verification/C2/readme.md)** | Shadow orbit error budget + Kaplan-Yorke bound | Chaos theory estimate |
| **[C3: Quantum](formal_verification/C3/readme.md)** | Grover search bound over configuration space Θ | Quantum search estimate |
| **[C4: Keystream](formal_verification/C4/readme.md)** | Deterministic extraction, domain separation | Plausibility argument |
| **[C5: Config](formal_verification/C5/readme.md)** | Configuration validation + estimated cardinality | Counting estimate |

---

## 4. Complete System Definition

### 4.1 Q32.64 Fixed-Point Arithmetic

All simulation arithmetic uses Q32.64 fixed-point representation on `i128`:
- **Representation:** `x_raw = round(x · 2^64)`, `1.0 = 2^64` raw
- **Addition/Subtraction:** Exact on `i128`
- **Multiplication:** `(a × b)_raw = (a_raw × b_raw) >> 64` — exact for bounded inputs
- **Division:** 192-iteration restoring division; error ≤ `|den_raw| >> 64 + 2` ULPs
- **Square root:** Binary digit-by-digit; error ≤ `(2 × result_raw) >> 64 + 3` ULPs
- **Physical domain:** Positions ∈ [−100, 100] AU, masses ∈ (0, 1] M☉, velocities ∈ [−100, 100] AU/yr

### 4.2 Simulation Loop

```
simulate(bodies, steps, dt, softening, g):
    for _ in 0..steps {
        verlet_step(bodies, dt, softening, g);
    }
```

- **Total steps:** `config.total_steps` (default 1,000,000)
- **Timestep:** Q32.64 from `DT = 0.01` (Verlet) or `DT = 0.001` (Euler)
- **Bodyguard (stability monitoring):** Runs in `simulate_with_monitoring()` at the KDF layer via Lyapunov estimation and ejection/collapse checks, not inline in the loop.

### 4.3 Entropy Extraction

```
seed_bytes = extract_shake256(
    &shake256_of(state.positions, state.velocities, G, softening)
);
```

SHAKE256 XOF → 2048-byte entropy pool (`kelvin-kdf/src/extractor.rs`).

### 4.4 Key Schedule

```
Pool → HKDF-SHA512(domain_sep, pool) → (KEY, NONCE, reseed_counter) → BLAKE3 → new_pool
```

---

## 5. L0: Safety Proofs

- **Lean proof:** [`proofs/lean/src/fixed_point/FixedPoint.lean`](proofs/lean/src/fixed_point/FixedPoint.lean) `no_overflow_*` theorems
- **Kani harness:** `proofs/kani/fixed_safety.rs`
- **Proves:** No panic, no overflow for `Fixed::{from_raw, add, sub, mul, div, sqrt}` and `compute_accelerations` under bounded inputs
- **Proof range:** Physical domain bounds (positions ±100 AU, masses 0–1 M☉, etc.)

## 6. L1: Functional Equivalence

- **Lean proof:** [`proofs/lean/src/fixed_point/FixedPoint.lean`](proofs/lean/src/fixed_point/FixedPoint.lean) `fromReal_correct`, `div_correctness`, `sqrt_correctness`
- **Kani harness:** `proofs/kani/fixed_equivalence.rs`
- **Proves:** Fixed-point arithmetic matches mathematical spec within dynamically scaled error bounds

## 7. L2: Composite Correctness

- **Lean proof:** [`proofs/lean/src/n_body/NewtonianGravity.lean`](proofs/lean/src/n_body/NewtonianGravity.lean) `action_reaction_law`, `force_direction_attractive`, `force_magnitude_formula`
- **Kani harness:** `proofs/kani/acceleration_proofs.rs`
- **Proves:** `compute_accelerations` satisfies Newton's laws (action-reaction, direction, magnitude)

## 8. L3: Pipeline Integrity

- **Lean proofs:** [`proofs/lean/src/dynamics/VerletIntegrator.lean`](proofs/lean/src/dynamics/VerletIntegrator.lean), [`proofs/lean/src/entropy_extraction/Shake256Extraction.lean`](proofs/lean/src/entropy_extraction/Shake256Extraction.lean), [`proofs/lean/src/key_schedule/HKDFDerivation.lean`](proofs/lean/src/key_schedule/HKDFDerivation.lean)
- **Kani harness:** `proofs/kani/pipeline_proofs.rs`
- **Proves:** `simulate_and_extract_seed` executes the correct number of steps, uses the correct domain separators, and produces consistent output

## 9. L4: Determinism

- **18 tests** in `tests/kelvin_tests/determinism.rs`
- **Proves:** Bit-identical results across SSE2, AVX, AVX2 for Verlet and Euler integrators

---

## 10. C1-C5 Gap Documentation

See [`formal_verification/PROOF_INDEX.md`](formal_verification/PROOF_INDEX.md) for the complete mapping of all 23 gaps to their Lean and Kani proofs.

| Section | File(s) | Description |
|--------|---------|-------------|
| **C1: Information Loss** | `formal_verification/C1/readme.md`, `gap{1..7}_*.md` | Fixed-point irreversibility |
| **C2: Lyapunov** | `formal_verification/C2/readme.md`, `gap{1..6}_*.md` | Chaos certification |
| **C3: Quantum Hardness** | `formal_verification/C3/readme.md`, `gap{1..3}_*.md` | Grover search bounds |
| **C4: Keystream** | `formal_verification/C4/readme.md`, `gap{1..3}_*.md` | Indistinguishability |
| **C5: Config Space** | `formal_verification/C5/readme.md`, `gap{1..4}_*.md` | Cardinality estimation |

---

## References

- Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980). "Lyapunov Characteristic Exponents for Smooth Dynamical Systems and for Hamiltonian Systems." *Meccanica*, 15, 9–20.
- Kaplan, J. L., & Yorke, J. A. (1979). "Chaotic behavior of multidimensional difference equations." *Functional Differential Equations and Approximation of Fixed Points*, 204–227.
- Poincaré, H. (1899). *Les Méthodes Nouvelles de la Mécanique Céleste*, Vol. 3.
- Shannon, C. E. (1949). "Communication Theory of Secrecy Systems." *Bell System Technical Journal*, 28(4), 656–715.

---

## Building All Proofs

```bash
cd proofs/lean
lean --init mathlib
lean build src/KelvinFormalVerif.lean
```

This builds all 7 Lean proof files which cover the mathematical specifications for all gaps.
