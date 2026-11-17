# TODO List

## 1. C2 – Lyapunov Spectrum – Full Kani proof
**Goal:** Replace the current “justified” status for `gap3_q3264_lyapunov_spectrum.md` with a complete Kani‑verified proof that the Q32.64 Lyapunov spectrum maps correctly from the floating‑point computation.
**Steps:**
1. Review the existing mathematical derivation in `documentation/formal_verification/C2/gap3_q3264_lyapunov_spectrum.md`.
2. Write a Kani harness `verify_lyapunov_spectrum.rs` that:
   - Generates random orbital configurations (`i128` fixed‑point values).
   - Computes the Lyapunov exponents using the Q32.64 implementation.
   - Computes the same exponents using a high‑precision reference (e.g., `rug` or `f64` with arbitrary precision) and compares within the error bound.
3. Add the harness to `proofs/kani/` and ensure it compiles (`cargo kani -p kelvin-core`).
4. Update the gap table in `C2/readme.md` to mark status **✅ RESOLVED**.
5. Run CI to verify the new proof passes.

**Owner:** verification team.

## 2. C4 – Keystream Indistinguishability – Formal security bound
**Goal:** Provide a rigorous, machine‑checked reduction showing that the keystream advantage is bounded by `negl(n) + 2⁻¹²⁸` (instead of the mixed bound currently noted).
**Steps:**
1. Clarify the intended bound in `documentation/formal_verification/C4/readme.md` and update the scope note.
2. Write a formal proof sketch in Lean/Coq (or as a Kani harness if feasible) that:
   - Replaces SHAKE256 with a random oracle and quantifies the negligible loss.
   - Uses the configuration‑space size from C5 and the Grover lower bound from C3.
   - Derives the final advantage bound `2⁻¹²⁸`.
3. Add the proof file `c4_security_bound.proof` in `documentation/formal_verification/C4/`.
4. Update the “All Gaps Resolved” table to reflect the new proof status.
5. Add a CI step that checks the proof file compiles (e.g., `lean --make` or `coqc`).

**Owner:** cryptographic analysis lead.

## 3. C5 – Full pipeline side‑channel hardening – Kani proof
**Goal:** Close the “PARTIAL” side‑channel hardening entry in `COMPLIANCE.md` by proving the entire encryption/decryption pipeline runs in constant time and leaks no secret information.
**Steps:**
1. Identify all secret‑dependent branches in the pipeline:
   - Extraction (`extract_shake256.rs`), key schedule (`kelvin-kdf/src/`), stream‑cipher modes.
2. Write a Kani harness `verify_side_channel.rs` that:
   - Executes the full `encrypt` and `decrypt` commands on randomly generated inputs.
   - Uses Kani’s `dudect` integration or custom timing instrumentation to assert that execution time does not vary with secret data.
3. Add the harness to `proofs/kani/` and run `cargo kani`.
4. Update `C5/readme.md` gap table to mark the side‑channel gap as **✅ RESOLVED**.
5. Ensure the CI badge for side‑channel proof appears in the README.

**Owner:** security engineering.

## 4. C1 – Information‑Loss aggregation – End‑to‑end bound
**Goal:** Derive a compositional theorem that aggregates per‑step information‑loss (from C1) into a concrete bound on the adversary’s advantage for the whole simulation.
**Steps:**
1. Formalise the per‑step loss `k_step ≥ 40 bits` (as in `C1/readme.md`).
2. Prove mathematically (and optionally in a proof assistant) that after `S` steps the total loss is `k_total = S * k_step` capped by the max entropy `H_max`.
3. Translate the result into a Kani‑compatible harness `verify_information_loss_total.rs` that checks the bound for a range of `S` values.
4. Update `C1/readme.md` with the new theorem and mark the gap as **✅ RESOLVED**.
5. Document the impact on the overall security claim in `COMPLIANCE.md`.

**Owner:** formal methods researcher.

## 5. C3 – Quantum hardness reduction – Kani‑checked reduction
**Goal:** Produce a machine‑checked reduction from a quantum adversary’s advantage to Grover’s lower bound, thereby removing the “proof‑sketch” tag.
**Steps:**
1. Define the oracle model in a Kani harness (`quantum_oracle.rs`) that abstracts the SHAKE256‑based keystream function.
2. Implement the reduction proof:
   - Show that any quantum algorithm distinguishing the keystream must query the oracle on a state that corresponds to a configuration `C`.
   - Use Zalka’s optimality theorem (formalised in Coq/Lean or as an external lemma) to bound the number of queries.
3. Connect the reduction to the configuration‑space size proven in C5.
4. Add the harness `verify_c3_quantum_reduction.rs` to `proofs/kani/` and ensure it compiles.
5. Update `C3/readme.md` gap table to **✅ RESOLVED** and note the new proof artifact.

**Owner:** quantum‑cryptography specialist.

## 6. C6 – Finite‑precision periodicity detection (address Critique #6)
**Goal:** Implement and formally verify a method to detect the simulation period and establish a concrete lower bound.
**Steps:**
1. Add `kelvin-core/src/period_detection.rs` implementing Floyd’s cycle‑finding on hashed simulation states.
2. Write Kani harness `verify_periodicity.rs` that runs the detector on random configurations and asserts no repeat before `MIN_STEPS` (e.g., 1 000 000).
3. Add benchmark script `scripts/measure_period.sh` that logs observed periods to `documentation/formal_verification/C6/period_results.md`.
4. Create `documentation/formal_verification/C6/README.md` summarising the method and the empirical guarantee.
5. Update `COMPLIANCE.md` with a new row under “Production hallmarks” → “Finite‑precision periodicity” indicating the current empirical guarantee and pending formal bound.
6. Track this work in the TODO list.

**Owner:** numerical analysis / formal methods researcher.
