# Kelvin Documentation Review

> **Scope:** All root-level and `documentation/` markdown files  
> **Reviewed:** 2026-05-31  
> **Severity key:** 🔴 Correction needed · 🟡 Enhancement suggested · 🔵 Clarification needed

---

## 1. README.md

### 🔴 Corrections

**Line 150 — Incorrect crate count**
> "Kelvin is organized as a Rust workspace with 22 crates"

The architecture block lists 14 named items (6 top-level crates + 8 test sub-crates). The actual count does not obviously reach 22. This number should be verified from `Cargo.toml` and updated, or the wording changed to "multiple crates".

**Lines 33–41 — Performance numbers inconsistent with bench_comparative.md**
The mode comparison table shows KelvinQuantum at **512 MB/s**, but `bench_comparative.md` (which is the authoritative benchmark) shows **438 MB/s** for a 1 MiB buffer. The README throughputs appear to be from a different benchmark run. Consider harmonising these or noting which measurement each refers to.

**Line 4 — Incorrect licence file link**
```markdown
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](licence.md)
```
The badge and links throughout the file point to `licence.md` (British spelling) for both Apache-2.0 and MIT licences, and also say:
> "Apache License, Version 2.0 ([LICENSE-APACHE](licence.md)…"  
> "MIT license ([LICENSE-MIT](licence.md)…"

Both licences map to the same file, which makes it appear there is only one licence document. Separate `LICENSE-APACHE` and `LICENSE-MIT` files are conventional for dual-licensed Rust crates. Additionally, `CONTRIBUTING.md` (line 32) states the licence is **CC-BY-NC-SA-4.0**, which directly contradicts the README's Apache-2.0 / MIT dual licence. This is a critical inconsistency.

**Line 50 — Prior art date may be wrong**
> "Chai et al. (2025)"

Confirm whether this paper is from 2025 or has a different publication date — it is referenced as "first known n-body chaotic image encryption" but should be verified against the REFERENCES.bib entry.

### 🟡 Enhancements

**Missing Rust version requirement**
The Usage Guide requires Rust 1.70+, but the README's Quick Start section omits any minimum version. Add it to the Prerequisites or Installation section.

**Missing `--release` flag note in Quick Start (lines 20–29)**
The Quick Start uses `cargo run -p kelvin-cli --`, which builds in debug mode. Add a note or use `cargo run --release` since simulation is CPU-intensive and debug builds would be very slow.

**Formal Verification table — L4 description**
> "Bit-identical results across platforms"

Consider adding that "18 determinism tests cover SSE2, AVX, AVX2, and NEON (aarch64)" as stated elsewhere — this makes the claim more concrete.

### 🔵 Clarifications

**Line 89 — "Unlimited" keystream in SHAKE256 context**
> "no quantum algorithm can shortcut the simulation"

While true for the n-body simulation, SHAKE256 itself is an infinite-output XOF constrained only by input — the statement would be clearer if it explicitly separated the two components (simulation + SHAKE256 extraction).

**Line 14 — V1 vs OTP modes mixing**
The introduction mentions both AEAD (V1) and OTP (V2/V3/H/Prism/Split/Flare) in the same sentence. A brief parenthetical clarifying that V1 is *not* an OTP mode would avoid confusion (V1 uses ChaCha20Poly1305, not XOR-based OTP).

---

## 2. CONTRIBUTING.md

### 🔴 Corrections

**Line 32 — Licence contradiction**
> "By contributing, you agree that your contributions will be licensed under the project's current license (CC-BY-NC-SA-4.0)."

The README and `CITATION.cff` show conflicting licences:
- README: Apache-2.0 / MIT (dual licence)
- CONTRIBUTING.md: CC-BY-NC-SA-4.0
- CITATION.cff: CC-BY-NC-SA-4.0

This **must** be resolved — the CLA/contribution licence terms must match the project licence. If the project is MIT/Apache-2.0, the contributing text must say so. If it is CC-BY-NC-SA-4.0, the README badges and `licence.md` links must reflect this.

### 🟡 Enhancements

**Missing link to SECURITY.md for security-sensitive changes**
The "Security-Sensitive Changes" section could link directly to SECURITY.md for the vulnerability reporting process.

**Missing `cargo deny` in test instructions**
The production readiness plan shows `cargo-deny` is integrated into CI (`deny.toml` exists), but it is not mentioned in CONTRIBUTING.md's test instructions. Add it:
```bash
cargo deny check
```

**Missing `--all-features` note**
Some features (e.g., `subtle-ct`, `failpoints`) are feature-gated. Contributors working on those areas should be directed to test with `--all-features`.

---

## 3. SECURITY.md

### 🔴 Corrections

**Line 57 — Authentication claim incorrectly describes authenticated modes**
> "The standard authenticated modes (V1 AEAD, V3/H with BLAKE3-keyed MAC) use classical cryptography for authentication."

This partially contradicts other documentation: the `--auth` flag uses **KMAC128** (NIST SP 800-185, based on Keccak), not "BLAKE3-keyed MAC". The production readiness plan says BLAKE3-keyed MAC was implemented, but `usage.md` (line 178) says KMAC128. These need to be reconciled — check `kelvin/src/authenticated.rs` to determine which is actually in use.

**Line 43 — Wrong author name for BLAKE3**
> "BLAKE3 (Baish et al., 2020)"

The correct authors are **O'Connor, Aumasson, Neves, Wilcox-O'Hearn** (2020). "Baish et al." does not appear to be correct. The `otp_bulletproof.md` references list "Aumasson, J. P., et al. (2013)" which is BLAKE2, not BLAKE3.

**Line 59 — Incorrect PQ signature module name**
> "The optional HAWK-512 module (Phase V) provides post-quantum digital signatures"

The project already uses **ML-DSA-65** (FIPS 204) for post-quantum signatures (implemented). HAWK-512 is listed as a *future* phase in the production readiness plan. The "standard authenticated modes" section should clarify that KMAC128 (Keccak-based) authentication already resists quantum MAC forgery differently from classical MACs.

### 🟡 Enhancements

**Missing coverage of V1 mode authentication**
The limitations section discusses authentication for V3/H modes but does not note that V1 (ChaCha20Poly1305) already provides quantum-resistant authentication (Poly1305 with a 256-bit ChaCha20-derived key). Clarify that V1 has better authentication posture than V3/H.

**Assumption 2 — Lyapunov time description could be clearer**
> "A Lyapunov time of ~1443 steps means the system doubles its trajectory divergence every ~1443 steps."

Lyapunov time is the time for divergence to grow by factor *e* (Euler's number ≈ 2.718), not by a factor of 2. "Doubling" is technically the half-Lyapunov time. Consider: "the system amplifies trajectory divergence by factor *e* every ~1443 steps."

---

## 4. THREAT_MODEL.md

### 🔴 Corrections

**CRLF line endings (cosmetic but inconsistent)**
The file uses Windows `\r\n` line endings while most other files use Unix `\n`. This causes visible differences in `git diff` and should be normalised (`.gitattributes` should already handle this).

**Line 54 — Timing variation in `verlet_step` description**
> "Timing variations of |t| ≈ 75 have been measured in dudect benchmarks."

The production readiness plan §1.3 notes this is "a benchmark artifact" and explains that the variation comes from `vec![]` allocation + state differences. The threat model presents this bare number without context, making it seem like a genuine side-channel. Add the same clarifying note: this is a benchmark artifact; `compute_accelerations` passes independently (|t| < 5), and the simulation runs before any keystream is produced.

**Lines 103–104 — Quantum resistance column for Argon2id and scrypt is incorrect**
| Property | Kelvin | Argon2id | scrypt | HKDF |
| **Quantum resistance** | 128-bit (SHAKE256) | None (SHA-256) | None (SHA-256) | None (SHA-256) |

Argon2id uses BLAKE2b (not SHA-256), and scrypt uses PBKDF2-SHA256 internally but its memory-hard structure is a different primitive. Marking them as "None (SHA-256)" is misleading — their quantum resistance is limited by their underlying hash function, but the hash names are wrong. Suggest "Partial (BLAKE2b)" for Argon2id and "Limited (SHA-256 via PBKDF2)" for scrypt.

### 🟡 Enhancements

**Missing threat: key config file compromise**
The security recommendations section (§6) discusses config reuse but does not address the scenario where the `key.json` file is compromised on disk. Add a recommendation: "Store orbital configurations encrypted at rest using OS keychain facilities or hardware security modules."

**Section 3.3 — Security Levels: add post-quantum bound**
The table shows raw keyspace (2¹²⁸⁷ etc.) but doesn't mention the effective 128-bit post-quantum bound imposed by SHAKE256 via Grover's. A footnote clarifying that the practical post-quantum security is bounded by SHAKE256's 128-bit Grover resistance (not by the keyspace) would be accurate and educational.

---

## 5. documentation/usage.md

### 🔴 Corrections

**Lines 2–3 — CLI flag inconsistency for Euler/Verlet**
> "Default: Verlet integration (symplectic, energy-conserving)"

But `Euler_vs_Verlet.md` (line 3) says:
> "Use `--verlet` to select Verlet integration; omit it for the default Euler."

And `usage.md` line 116 says:
> "See `Euler_vs_Verlet.md` for the full theoretical analysis."

These two documents contradict each other on which is the **default** integration method. This needs to be resolved by checking the actual CLI implementation (`kelvin-cli/src/main.rs`).

**Line 70 — Missing closing backtick in code block**
```
### Identity (Asymmetric Keys)
...
./kelvin identify --config my_secret.json --ecc --kem

### Authenticated Encryption (`--auth`)
```
The `identify` command block is missing its closing ` ``` ` fence. This will render the next section as part of the code block.

**Line 245 — `remaining_safe_bytes()` returns unit incorrectly described**
> "This returns `remaining_keys × 4 GiB` (conservative estimate)."

The V1 mode stores approximately 4 GiB per key via ChaCha20 (32-bit counter × block size). For OTP modes (V3/H), the limit is the key schedule — not the ChaCha20 counter. Clarify that the 4 GiB figure applies to the V1 (ChaCha20Poly1305) cipher, and that OTP modes (V3/H) can encrypt larger files per key.

**Line 11 — Incorrect Rust version requirement**
> "Rust (1.70 or later)"

The README Installation section shows no minimum version. The production readiness plan uses features that may require a more recent Rust edition (2021 edition, various stabilized features). Verify the actual MSRV and state it consistently.

### 🟡 Enhancements

**Missing section on key rotation / re-keying**
The guide explains `SeedExhausted` errors but doesn't provide a practical pattern for key rotation — i.e., how to generate a new `OrbitalConfig` and seamlessly transition encryption. Add a small example or link to the relevant pattern.

**V2 security note should mention `--auth` more prominently**
The security notes for V2 (line 425) say "V2 is a pure XOR stream cipher — no authentication. Use a MAC for integrity." but don't give the actual CLI flag (`--auth`) or API wrapper (`KelvinStreamingAuthenticated`) in that section — they are explained elsewhere but a forward reference here would improve discoverability.

**Prism/Split/Flare — add cross-reference to homomorphic doc**
The Prism section links to `homomorphic_cryptosystem.md` at the end, but it would be useful to also link it in the introduction.

---

## 6. documentation/otp_bulletproof.md

### 🔴 Corrections

**Duplicate section heading (line 62)**
> `## 3. Attack Vector Analysis`

The document already has `## 3. The Four Pillars of the OTP Claim` at line 50. Having two `## 3.` sections will break markdown renderers and table-of-content generators. Renumber to `## 4. Attack Vector Analysis` and adjust subsequent sections accordingly.

**BLAKE3 reference (line 174)**
> "Aumasson, J. P., et al. (2013). 'BLAKE2: simpler, smaller, fast as MD5.'"

This references BLAKE**2** (2013), not BLAKE3. BLAKE3 was published in 2020 by O'Connor, Aumasson, Neves, and Wilcox-O'Hearn at USENIX Security 2021. Either add the correct BLAKE3 reference or remove this entry (BLAKE2 is not used in the current implementation).

**Line 42 — Preimage resistance for SHAKE256 stated incorrectly**
> "Distinguishing SHAKE256 output from random requires breaking the Keccak sponge — a problem with no known solution better than brute force (2^512 preimage resistance)."

SHAKE256 has **256-bit** preimage resistance (not 2^512). The capacity of SHAKE256 is 512 bits, and preimage resistance for a hash/XOF is half the capacity = 256 bits. The `2^512` figure is incorrect. The correct value is 2^256 classical / 2^128 quantum. (The 2^512 figure would apply to SHA3-512.)

### 🟡 Enhancements

**The "Bulletproof" title should acknowledge caveats more visibly**
The document is thorough in its honest qualification (§2, "The Honest Qualification") but the overall framing in the title and executive claim may be over-confident for an experimental system. Consider adding an explicit disclaimer box at the top referencing the lack of formal cryptanalysis.

---

## 7. documentation/formal_verification.md

### 🔵 Clarifications

**Section 5 — Conjecture C5 uses LaTeX inside markdown tables**
> `$\lvert\Theta_5\rvert \ge 2^{1920}$`, `$H_{\min} \ge 1800$ bits`

These LaTeX expressions render correctly only in environments with MathJax (e.g., GitHub renders basic LaTeX in some contexts but not markdown tables reliably). Consider providing plain-text equivalents: "|Θ₅| ≥ 2^1920, H_min ≥ 1800 bits".

**Section 6 — Docker command inconsistency**
> `docker compose -f docker/docker-compose.yml run kani bash -c "cd /kelvin && docker/run-kani.sh"`

But the production readiness plan (§3.1) shows:
> `docker compose -f docker/docker-compose.yml run cross`

The `kani` service name is used for Kani verification; `cross` for cross-compilation. The comment "via Docker" in the empirical validation section (C1–C5 cargo commands) is misleading — those run directly with `cargo run`, not inside Docker.

---

## 8. documentation/security_assumptions.md

### 🟡 Enhancements

**Assumption 1 — Poincaré reference year is wrong**
> "The N-body problem (N ≥ 3) has no closed-form analytical solution (Poincaré, 1889)."

But at the bottom of the document (references section):
> "Poincaré, H. (1889). 'Sur le problème des trois corps…' Acta Mathematica, 13, 1–270."

The famous *Les Méthodes Nouvelles* (where the non-integrability is fully elaborated) is **1899**, Vol. 3. The 1889 paper is the earlier memoir. Both references appear in different documents with different years (THREAT_MODEL.md cites 1899; this file cites 1889). Decide which is the canonical citation and use it consistently throughout.

**Missing assumption about HKDF-SHA512**
The document lists 4 assumptions (N-body OWF, fixed-point determinism, SHAKE256, Lyapunov horizon) but omits an explicit assumption about **HKDF-SHA512** security. HKDF is used in Phase 3 of the key schedule, and the key schedule's security depends on SHA-512's PRF properties. This should be a 5th assumption for completeness.

---

## 9. documentation/Euler_vs_Verlet.md

### 🔴 Corrections

**Lines 74–76 — Code snippet uses floating-point, not fixed-point**
```rust
const DT: f64 = 0.001;
const G: f64 = 1.0;
const EPSILON: f64 = 1e-6;
```

The implementation snippets show `f64` arithmetic, but Kelvin's production code uses Q32.64 fixed-point arithmetic exclusively. This appears to be pseudo-code or an early prototype. The docstring should clarify these are illustrative, not the actual production implementation. In production, `DT` is `Fixed::from_raw(...)` etc.

**Line 3 — Default integration method inconsistency**
> "Use `--verlet` to select Verlet integration; omit it for the default Euler."

But `usage.md` repeatedly says "Default: Verlet integration." See the contradiction flagged under `usage.md` above — one of these must be wrong.

### 🟡 Enhancements

**Performance table (lines 56–62) — units for dt are not comparable**
The table compares Euler at dt=0.001 vs Verlet at dt=0.01. This is a 10× difference in timestep. The comparison of "time per step" (80 ns vs 100 ns) is misleading without noting that the simulation covers different physical time per step. A note explaining that Euler needs smaller dt for stability, so equal "physical time" coverage requires 10× more steps, would make this clearer.

---

## 10. documentation/keyspace_analysis.md

### 🟡 Enhancements

**Line 187–192 — RNG seed entropy cap section needs clarification**
> "While the expression space is 2^1113, the observable keyspace is capped by the RNG seed entropy: 2^256 distinct seeds."

This is an important security observation but may alarm readers. Add a clarifying sentence: "This is equivalent to the 256-bit security of a standard symmetric cipher (AES-256) — adequate for all practical purposes. The 2^1113 expression space provides headroom for structural diversity even if the underlying RNG were weakened."

**Line 200 — "2^256 | ≥ 2^256 | ✓ Marginal" framing**
Calling 256-bit security "Marginal" is potentially misleading. 256-bit symmetric key strength is considered post-quantum secure (128 bits after Grover). Consider replacing "Marginal" with "Adequate (matches AES-256 security level)".

---

## 11. documentation/quantum_analysis.md

### 🔴 Corrections

**Line 10 — Incorrect quantum security for SHAKE256**
> "Entropy Extractor (SHAKE256) | SHAKE256 (XOF) | Quantum-Resistant | **256-bit (Grover)**"

Grover's algorithm provides a quadratic speedup, halving the effective security. SHAKE256 provides **128-bit** post-quantum security (not 256-bit). 256 bits is the *classical* security level. This appears in the executive summary table and is repeated incorrectly elsewhere. The correct table entry should read "128-bit (Grover)" — consistent with Section 2.1 and `otp_bulletproof.md`.

**Line 51 — ML-DSA-65 hardness description**
> "Based on the Module Learning with Errors (M-LWE) problem"

ML-DSA (CRYSTALS-Dilithium) is based on **Module Learning with Errors (MLWE) and Module Short Integer Solution (MSIS)**, not just M-LWE. The description is partially correct but incomplete for a security document.

### 🔵 Clarifications

**Section 2.3.1 — "Deep Physical Binding" is not a standard term**
The term "Deep Physical Binding" is used without definition or reference. Define it explicitly: "the practice of including simulation-derived quantities (G, ε, force vectors, step counter) in the hash input alongside orbital state, to prevent adversaries from modelling partial system state".

---

## 12. CITATION.cff

### 🔴 Corrections

**Line 12 — Licence inconsistency**
> `license: "CC-BY-NC-SA-4.0"`

This contradicts the README's Apache-2.0/MIT dual licence. This must be resolved — it is the machine-readable licence declaration and affects how tools (Zenodo, CFF parsers) report the licence.

**Line 20 — Abstract mentions SHA3-512 and ChaCha20 but system has evolved significantly**
> "It combines Q32.64 fixed-point arithmetic, a symplectic Verlet integrator, Lyapunov time estimation, SHA3-512 entropy extraction, and a ChaCha20 stream cipher."

The abstract is outdated: the primary extractor is now **SHAKE256** (SHA3-512 is used only for legacy seed), and the ciphertext modes include SHAKE256 XOR (V2/V3/H) alongside ChaCha20 (V1). The dominant modes (Quantum, Photon, Prism, Split, Flare) do not use ChaCha20. Update the abstract.

**Line 10 — `date-released: 2026`** should include a full date (e.g., `2026-05-01`) per CFF 1.2 specification, which requires ISO 8601 format (`YYYY-MM-DD`).

**Line 79 — CryptoChaos author is incorrectly listed as "Harvard University"**
> `name: "Harvard University"` / `title: "CryptoChaos: A Hybrid Chaos-Based..."`

Song et al. (2025) "CryptoChaos" (arXiv:2504.08618) is authored by individual researchers, not Harvard University. Harvard may be an affiliation but should not be the author. Use the actual author names from the paper.

---

## 13. production_readiness_plan.md

### 🟡 Enhancements

**Section 4.2 — "Disclosure Plan" is marked incomplete but SECURITY.md already covers it**
> - [ ] **Disclosure Plan**: Define how security advisories will be communicated to users.

SECURITY.md (§Disclosure Plan) already defines this (GitHub Security Advisories, release notes, 90-day timeline). Mark this item as `[x]` and link to SECURITY.md.

**Section 3.4 — Comparative benchmarking results already exist**
> - [ ] **Throughput Comparison**: Run `criterion` benchmarks comparing Kelvin modes against established libraries.

`bench_comparative.md` already shows the requested comparisons (KelvinQuantum vs AES-256-GCM vs ChaCha20-Poly1305, KelvinStreaming vs AES-256-CTR, X25519/Ed25519 keygen). This item should be at least partially checked.

**Section 1.3 — Note about `verlet_step` timing would benefit from a planned resolution**
The note about |t| ≈ 75 for `verlet_step`/`simulate` is documented as a "benchmark artifact" but there is no tracking item to properly investigate it. Consider adding a task: `- [ ] Investigate verlet_step timing variation: isolate vec![] allocation from simulation timing to confirm it is an artifact`.

---

## 14. documentation/nist_800_90b_report.md

### 🟡 Enhancements

**Section 3.1 — Test sample size mismatch in heading vs body**
> "Results from 1,048,576 bytes (1 MiB) of raw SHAKE256 XOR keystream"

But Appendix A says "Analyzing 1048576 bytes from: **keystream_1mb.bin**". The report methodology section (§2.1) says the test data is 1 GiB. The actual health test results are on 1 MiB, not 1 GiB as implied by §2. Clarify this prominently.

**Section 3.2 — "NIST ea_iid Results — Pending" should have a target date**
This section has been pending since the report date (2026-05-23). Add a target milestone date or link this to the Phase II/IV timeline in `production_readiness_plan.md`.

---

## 15. documentation/project_history.md

### 🟡 Enhancements

**Parenthetical typo**
> "(not a full time job;)" 

Should be "(not a full-time job)" — hyphenate the compound modifier and remove the semicolon.

**Line 10 — "(Using fast Euler approximation)"**
> "*   **Initial Conception**: The project began as a theoretical exploration of using the n-body problem for non-repeating keystream generation. (Using fast Euler approximation)"

This parenthetical is not in standard documentation style. Integrate it into the sentence: "…keystream generation, initially using Euler integration for speed."

---

## 16. Cross-Document Issues

### 🔴 Authentication mechanism contradiction

Multiple documents disagree on what authenticates the Chaos/Photon/Quantum modes:
- `SECURITY.md` says: "BLAKE3-keyed MAC"
- `usage.md` says: "KMAC128 (NIST SP 800-185)"  
- `production_readiness_plan.md` says: "BLAKE3-keyed MAC authenticated tagging (32-byte tag)"
- `otp_bulletproof.md` says: "KMAC128 (NIST SP 800-185)"

The CLI flag is `--auth` and the wrapper types are `KelvinStreamingAuthenticated`, `KelvinPhotonAuthenticated`, `KelvinQuantumAuthenticated`. **Verify from `kelvin/src/authenticated.rs`** which MAC is actually used, and update all documents to match.

### 🔴 Licence inconsistency

Three different licences are claimed across the documentation:
1. **Apache-2.0 / MIT** (dual licence) — README.md
2. **CC-BY-NC-SA-4.0** — CONTRIBUTING.md and CITATION.cff

This is a significant legal inconsistency. The project needs a single authoritative licence statement.

### 🔴 Default integration method inconsistency

- `Euler_vs_Verlet.md`: "omit [--verlet] for the **default Euler**"
- `usage.md`: "**Default: Verlet** integration (symplectic, energy-conserving)"

These directly contradict each other. Check `kelvin-cli/src/main.rs` to determine the actual default.

### 🟡 Poincaré year consistency

Different documents cite either 1889 or 1899 for Poincaré's foundational work:
- `security_assumptions.md` references: **1889** (the Acta Mathematica memoir)
- `THREAT_MODEL.md`: **1899** (*Les Méthodes Nouvelles*, Vol. 3)  
- `otp_bulletproof.md`: **1899**
- `formal_verification.md`: **1899**

Both works exist and are relevant. Choose one canonical citation and use it consistently, or cite both where appropriate (1889 for the original three-body memoir; 1899 for the full treatment of non-integrability).

### 🟡 SHAKE256 quantum security repeatedly stated incorrectly

At least `quantum_analysis.md` states SHAKE256 offers "256-bit (Grover)" quantum security. The correct figure is **128-bit quantum / 256-bit classical**. This error appears in the executive summary table — fix it there and ensure it doesn't appear elsewhere.

---

## Summary by Priority

| Priority | Count | Items |
|----------|-------|-------|
| 🔴 Must fix | 12 | Licence contradiction, auth mechanism conflict, default integrator conflict, SHAKE256 preimage claim, section numbering duplicate, BLAKE3 author name, BLAKE3 reference in otp_bulletproof, float code in Euler_vs_Verlet, CITATION.cff abstract, CITATION.cff date format, CITATION.cff CryptoChaos author, quantum_analysis SHAKE256 table |
| 🟡 Should fix | 14 | README crate count, benchmark inconsistency, README prerequisites, Euler/Verlet dt comparison note, keyspace "marginal" framing, NIST report section mismatch, ea_iid pending deadline, HKDF assumption missing, authentication prominence in usage.md, production plan disclosure/benchmark items, project history style, Lyapunov "doubling" description |
| 🔵 Consider | 4 | LaTeX in formal_verification tables, "Deep Physical Binding" definition, otp_bulletproof framing, SHAKE256 unlimited keystream clarification |
