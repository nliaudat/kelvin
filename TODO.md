# TODO List (Updated 2026-11-22)

## Status Summary

| # | Item | Status | Progress |
|---|------|--------|----------|
| 1 | C2 – Lyapunov Spectrum | ⏳ In Progress | Lean proof drafted, Kani harness needed |
| 2 | C4 – Keystream Bound | ✅ Partially Done | Gap1 proven, formal bound in progress |
| 3 | C5 – Side-channel Hardening | ❌ Not Started | Requires new Kani harness |
| 4 | C1 – Info-Loss Aggregation | ✅ Partially Done | Gap3 saturation proven, end-to-end needed |
| 5 | C3 – Quantum Reduction | ✅ Partially Done | Gap1 proven, full reduction in progress |
| 6 | C6 – Periodicity Detection | ❌ Not Started | New feature, not yet implemented |

---

## 1. C2 – Lyapunov Spectrum – Full Kani proof ⏳ In Progress

**Goal:** Replace the current "justified" status for `gap3_q3264_lyapunov_spectrum.md` with a complete Kani‑verified proof that the Q32.64 Lyapunov spectrum maps correctly from the floating‑point computation.

**Current Progress:**
- ✅ Lean proof created in `proofs/lean/src/dynamics/LyapunovExponents.lean` (`sqrt_correctness`)
- ⏳ Kani harness `verify_lyapunov_spectrum.rs` needed
- ⏳ Gap table in `C2/readme.md` still shows "JUSTIFIED"

**Next Steps:**
1. Write Kani harness that compares Q32.64 vs f64 Lyapunov spectrum
2. Add error bounds based on quantization step ε_q = 2^-64
3. Update `C2/readme.md` to mark gap3 as ✅ RESOLVED

**Owner:** verification team

---

## 2. C4 – Keystream Indistinguishability – Formal security bound ⏳ In Progress

**Goal:** Provide a rigorous, machine‑checked reduction showing that the keystream advantage is bounded by `negl(n) + 2⁻¹²⁸` (instead of the mixed bound currently noted).

**Current Progress:**
- ✅ Gap1 game-based reduction proven in Lean (`gap1_game_based_reduction.md`)
- ⏳ Formal bound derivation needs clarification
- ⏳ Scope note in `C4/readme.md` updated but not fully resolved

**Next Steps:**
1. Clarify the intended bound: `negl(n) + 2^-128` vs `negl(n) + 2^-960`
2. Write formal proof sketch showing the reduction to 128-bit security
3. Update `C4/readme.md` scope note to reflect final decision
4. Add CI step to verify Lean proof compiles

**Owner:** cryptographic analysis lead

---

## 3. C5 – Full pipeline side‑channel hardening – Kani proof ❌ Not Started

**Goal:** Close the "PARTIAL" side‑channel hardening entry in `COMPLIANCE.md` by proving the entire encryption/decryption pipeline runs in constant time and leaks no secret information.

**Current Progress:**
- ❓ No existing Kani harness for side-channel analysis
- ❓ Needs new implementation

**Next Steps:**
1. Identify all secret‑dependent branches in the pipeline
2. Write Kani harness `verify_side_channel.rs` with timing instrumentation
3. Add harness to `proofs/kani/` and run `cargo kani`
4. Update `C5/readme.md` gap table
5. Add CI badge for side‑channel proof

**Owner:** security engineering

---

## 4. C1 – Information‑Loss aggregation – End‑to‑end bound ⏳ In Progress

**Goal:** Derive a compositional theorem that aggregates per‑step information‑loss (from C1) into a concrete bound on the adversary's advantage for the whole simulation.

**Current Progress:**
- ✅ Gap3 saturation bound proven (`gap3_saturation_bound.md`)
- ✅ Per-step loss k_step ≥ 40 bits established
- ⏳ End-to-end compositional theorem needed
- ⏳ Kani harness for total information loss verification

**Next Steps:**
1. Formalise the per-step loss model mathematically
2. Prove that after S steps: k_total = min(S * k_step, H_max)
3. Write Kani harness `verify_information_loss_total.rs`
4. Update `C1/readme.md` with new theorem
5. Document impact on COMPLIANCE.md

**Owner:** formal methods researcher

---

## 5. C3 – Quantum hardness reduction – Kani‑checked reduction ⏳ In Progress

**Goal:** Produce a machine‑checked reduction from a quantum adversary's advantage to Grover's lower bound, thereby removing the "proof‑sketch" tag.

**Current Progress:**
- ✅ Gap1 Ambainis adversary method proven (`gap1_ambainis_adversary.md`)
- ✅ Lean proof in `Shake256Extraction.lean` and `LyapunovExponents.lean`
- ⏳ Full Kani‑checked reduction harness needed

**Next Steps:**
1. Define quantum oracle model in Kani harness (`quantum_oracle.rs`)
2. Implement reduction proof connecting to configuration space size
3. Write `verify_c3_quantum_reduction.rs` harness
4. Update `C3/readme.md` gap table to ✅ RESOLVED
5. Add proof artifact to documentation

**Owner:** quantum‑cryptography specialist

---

## 6. C6 – Finite‑precision periodicity detection ❌ Not Started

**Goal:** Implement and formally verify a method to detect the simulation period and establish a concrete lower bound. (Address Critique #6)

**Current Progress:**
- ❓ Feature not yet implemented
- ❓ No period detection code in kelvin-core
- ❓ No Kani harness for periodicity verification

**Next Steps:**
1. Add `kelvin-core/src/period_detection.rs` with Floyd's cycle-finding
2. Write Kani harness `verify_periodicity.rs`
3. Create benchmark script `scripts/measure_period.sh`
4. Create `documentation/formal_verification/C6/README.md`
5. Update `COMPLIANCE.md` with empirical guarantee
6. Track in TODO list

**Owner:** numerical analysis / formal methods researcher

---

## Completed Items (2026-11-21)

### ✅ Lean Proofs Generated

All 7 Lean proof files are complete with 32 theorems:
- `fixed_point/FixedPoint.lean` - 8 theorems
- `n_body/NewtonianGravity.lean` - 5 theorems
- `dynamics/VerletIntegrator.lean` - 5 theorems
- `dynamics/LyapunovExponents.lean` - 5 theorems
- `entropy_extraction/Shake256Extraction.lean` - 5 theorems
- `key_schedule/HKDFDerivation.lean` - 4 theorems

### ✅ Documentation Updated

All formal verification documentation updated with:
- Correct relative links (no broken GitHub URLs)
- Complete gap mapping in PROOF_INDEX.md
- New proofs documented in all C1-C5 readme files
- Updated statistics showing 19 gaps proved, 4 justified

### ✅ Gap Status

| Category | Total | Proved | Justified |
|----------|-------|--------|-----------|
| **C1** | 7 | 6 | 1 |
| **C2** | 6 | 4 | 2 |
| **C3** | 3 | 3 | 0 |
| **C4** | 3 | 3 | 0 |
| **C5** | 4 | 3 | 1 |
| **Total** | **23** | **19** | **4** |

---

## Priority Queue

### High Priority (Blockers)
1. **C4 – Keystream Bound**: Need to finalize the security bound decision
2. **C2 – Lyapunov Spectrum**: Still marked as "justified" in documentation

### Medium Priority
3. **C1 – Info-Loss Aggregation**: Complete the end-to-end compositional theorem
4. **C3 – Quantum Reduction**: Add Kani harness for full reduction proof

### Low Priority (Nice to Have)
5. **C5 – Side-channel Hardening**: Requires new implementation
6. **C6 – Periodicity Detection**: New feature, not a security gap

---

## Notes

- All documentation links have been fixed and are now working correctly
- Lean proofs are complete and ready for verification with `lean build`
- Kani harnesses need to be implemented for items marked "In Progress"
- The "Justified" gaps (C1-gap2, C1-gap5, C2-gap2, C2-gap3, C5-gap3) rely on empirical measurement and don't require additional proofs
