# Formal Verification — Kelvin Cryptosystem

> **Status (2026-11-23):**
> - **Lean:** 50 theorems machine-checked in `proofs/lean` (self-contained, no
>   Mathlib; 23 primary + 27 legacy adapters). They cover integer arithmetic,
>   Verlet scheduling, pairwise momentum, domain framing, Kaplan–Yorke
>   arithmetic, and Floyd loop invariants.
> - **Kani:** 46 harnesses: 37 compiled into `kelvin-core` (8 in
>   `fixed_math.rs`, 29 included from `proofs/kani/`), 6 in `kelvin-kdf` and 3 in
>   `kelvin`. The remaining `proofs/kani/*.rs` files are
>   templates and are not compiled.
> - **C1–C5:** **none of the 23 security gaps is machine-checked.** Three have a
>   partial arithmetic artifact, five are empirical, fifteen are open. See
>   [`formal_verification/PROOF_INDEX.md`](formal_verification/PROOF_INDEX.md).
>
> The L0–L4 Kani proofs check code against specification for fixed-point
> arithmetic. The C1–C5 conjectures are empirical arguments and plausibility
> estimates — they are **not** formal security reductions.

## 1. Proof architecture

```
Lean (proofs/lean, zero dependencies)
├── FixedPoint.lean        → raw i128 arithmetic: overflow, div/mod
├── Integrator.lean        → kick-drift-kick scheduling, reversibility
├── Gravity.lean           → pairwise momentum cancellation
├── KeySchedule.lean       → index encoding, conditional uniqueness
├── Entropy.lean           → domain-separated framing
└── PeriodDetection.lean   → Floyd loop invariants
        │  (link by inspection — not a verified refinement)
        ▼
Rust (kelvin-core/src)
├── fixed_math.rs          → Q32.64 arithmetic (+ 8 #[cfg(kani)] harnesses)
├── body.rs                → Vec3, OrbitalBody
├── integrator.rs          → Verlet, Euler
├── information_loss.rs    → placeholder (returns steps * 1.0)
├── quantum.rs             → placeholder
├── reference.rs           → placeholder
└── period_detection.rs    → Floyd cycle finding (find_cycle)
```

There is **no** verified translation from the Lean model to the Rust code. The
correspondence is by inspection, and the models are deliberately coarse.

### 1.1 Code-level verification

The Kani strategy is described in
[`formal_verification/code_verification.md`](formal_verification/code_verification.md).
46 harnesses run: 8 in `kelvin-core/src/fixed_math.rs`, 29 included from
`proofs/kani/` by `kelvin-core/src/kani_harnesses.rs`, 6 in `kelvin-kdf` and 3
in `kelvin`. Run them with:

```bash
docker compose -f docker/docker-compose.yml run --rm kani --list
docker compose -f docker/docker-compose.yml run --rm kani
```

---

## 2. Executive summary

Kelvin derives a keystream from a deterministic fixed-point n-body simulation
followed by SHAKE256 extraction. The security claim is that this pipeline is a
**quantum-resistant one-way function**. That is a conjecture; it is not proved
here.

The intended chain of reasoning is:

```
C1 (k_step ≥ 40 bits/step) → information-loss estimate
  → C2 (λ > 0, Kaplan–Yorke bound) → chaos certification
  → C3 (Ω(2^960) Grover bound over Θ) → quantum search estimate
  → C5 (|Θ₅| ≥ 2^1920) → cardinality estimate
  → C4 (Adv(A) ≤ negl(n) + 2^-960) → distinguishing advantage estimate
```

Every link in that chain is either empirical or open.

> **Caveat on C4.** The `2⁻⁹⁶⁰` term comes from a Grover search over the
> theoretical configuration space ($2^{1920}$). The reachable keyspace is
> limited by the OS CSPRNG ($2^{256}$), and the Grover oracle may have
> exploitable structure. The effective post-quantum security, bounded by
> SHAKE256's Grover resistance, is **128-bit**. See
> `stream_cipher_security.md` §1.

---

## 3. Proof levels

| Level | What is checked | Where |
|-------|-----------------|-------|
| **L0: Safety** | No overflow for bounded add/sub | [`FixedPoint.lean`](../proofs/lean/src/KelvinProofs/FixedPoint.lean) + Kani `verify_add_no_overflow`, `verify_sub_no_overflow`, `verify_mul_range` |
| **L1: Functional eq.** | Division/remainder identities | [`FixedPoint.lean`](../proofs/lean/src/KelvinProofs/FixedPoint.lean) + Kani `verify_c1_division_remainder` |
| **L2: Composite** | Pairwise momentum cancellation | [`Gravity.lean`](../proofs/lean/src/KelvinProofs/Gravity.lean) |
| **L3: Pipeline** | Verlet scheduling, domain framing | [`Integrator.lean`](../proofs/lean/src/KelvinProofs/Integrator.lean), [`Entropy.lean`](../proofs/lean/src/KelvinProofs/Entropy.lean) |
| **L4: Determinism** | Bit-identical across targets | `tests/kelvin_tests/determinism.rs` |

These are implementation facts, not security proofs.

Empirical estimates and plausibility arguments (not proofs):

| Level | Description | Type |
|-------|-------------|------|
| [C1: Information Loss](formal_verification/C1/readme.md) | Fixed-point rounding irreversibility | Empirical measurement |
| [C2: Lyapunov](formal_verification/C2/readme.md) | Shadow-orbit error budget, Kaplan–Yorke | Chaos-theory estimate |
| [C3: Quantum](formal_verification/C3/readme.md) | Grover bound over $\Theta$ | Quantum-search estimate |
| [C4: Keystream](formal_verification/C4/readme.md) | Extraction, domain separation | Plausibility argument |
| [C5: Config](formal_verification/C5/readme.md) | Validation, estimated cardinality | Counting estimate |

---

## 4. System definition

### 4.1 Q32.64 fixed-point arithmetic

- **Representation:** `x_raw = round(x · 2^64)`, `1.0 = 2^64` raw, stored in `i128`
- **Addition/subtraction:** exact on `i128`
- **Multiplication:** `(a × b)_raw = (a_raw × b_raw) >> 64`
- **Division:** restoring division; bounded ULP error
- **Square root:** binary digit-by-digit; bounded ULP error
- **Physical domain:** positions ∈ [−100, 100] AU, masses ∈ (0, 1] M☉,
  velocities ∈ [−100, 100] AU/yr

### 4.2 Simulation loop

```
simulate(bodies, steps, dt, softening, g):
    for _ in 0..steps {
        verlet_step(bodies, dt, softening, g);
    }
```

- **Total steps:** `config.total_steps` (default 1,000,000)
- **Timestep:** from `DT` (Verlet) or `--euler` timestep
- **Stability monitoring:** `simulate_with_monitoring()` at the KDF layer

### 4.3 Entropy extraction

```
seed_bytes = extract_shake256(
    &shake256_of(state.positions, state.velocities, G, softening)
);
```

SHAKE256 XOF → 2048-byte entropy pool (`kelvin-kdf/src/extractor.rs`).

### 4.4 Key schedule

```
Pool → HKDF-SHA512(domain_sep, pool) → (KEY, NONCE, reseed_counter) → BLAKE3 → new_pool
```

---

## 5. L0: Safety

- **Lean:** `FixedPoint.add_no_overflow`, `sub_no_overflow`,
  `phys_add_fits_i128`, `phys_sub_fits_i128`.
- **Kani:** `verify_add_no_overflow`, `verify_sub_no_overflow`, `verify_mul_range`
  (`kelvin-core/src/fixed_math.rs`).
- **Scope:** physical-domain bounds. Multiplication is checked by Kani, not Lean.

## 6. L1: Functional equivalence

- **Lean:** `div_rem_identity`, `rem_bounds`.
- **Kani:** `verify_c1_division_remainder`.
- **Scope:** division/remainder identities. A full error-bound proof for `div`
  and `sqrt` against a real-valued specification does **not** exist.

## 7. L2: Composite correctness

- **Lean:** `Gravity.pair_momentum_cancel`, `accel_zero_mass`.
- **Kani:** `verify_c3_step_damping`, `verify_c3_two_step_dispersion`
  (division-rounding checks).
- **Scope:** pairwise momentum cancellation. Newton's third law in full vector
  form, and the force magnitude, are not formalised.

## 8. L3: Pipeline integrity

- **Lean:** `Integrator.step_fst`, `step_snd`, `step_reversible`,
  `simulate_zero`; `Entropy.domain_separation`;
  `KeySchedule.infoString_injective`.
- **Kani:** the `verify_c1_*` harnesses.
- **Scope:** update scheduling and framing. "Executes the correct number of
  steps" and "uses the correct domain separators" are checked by inspection.

## 9. L4: Determinism

- **18 tests** in `tests/kelvin_tests/determinism.rs`
- Checks bit-identical results across SSE2/AVX/AVX2 for Verlet and Euler.

---

## 10. C1–C5 gap documentation

See [`formal_verification/PROOF_INDEX.md`](formal_verification/PROOF_INDEX.md)
for the gap-by-gap mapping. Current totals: 0 proved, 3 partial, 5 empirical,
15 open.

| Section | Files | Description |
|---------|-------|-------------|
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
- Zalka, C. (1999). "Grover's quantum searching algorithm is optimal." *Phys. Rev. A* 60, 2746.

---

## Building the proofs

```bash
cd proofs/lean
lake build
```

Requires `elan`/`lake` with the toolchain in `lean-toolchain`. No Mathlib.
See [`proofs/lean/README.md`](../proofs/lean/README.md).

Kani:

```bash
docker compose -f docker/docker-compose.yml build kani
docker compose -f docker/docker-compose.yml run --rm kani
```
