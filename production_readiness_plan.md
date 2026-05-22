# Kelvin Production Readiness Plan

This document outlines the roadmap to transition the **Kelvin Cryptosystem** from its current "Experimental" status to a "Production-Ready" state. The goal is to reach a level of maturity comparable to industry-standard libraries like `ring`, `dalek-cryptography`, or `rust-crypto`.

---

## 1. Security Hardening & Verification

Security is the primary requirement for production readiness. We must move beyond "it passes unit tests" to "it is verified against classes of vulnerabilities."

### 1.1 Formal Verification
- [x] **Core Math Verification**: Use [Kani](https://model-checking.github.io/kani/) to formally verify that the fixed-point arithmetic (`Q32.64`) never overflows under valid orbital configurations. *(Completed 2026-05-22)*
    - Five proof harnesses implemented in `kelvin-core/src/fixed_math.rs`:
        1. `verify_add_no_overflow` — add never wraps for positions in [-100, 100] AU
        2. `verify_sub_no_overflow` — sub never wraps for positions in [-100, 100] AU
        3. `verify_mul_no_overflow` — mul splitting handles all products in [-100, 100] AU
        4. `verify_div_no_panic` — div never panics for G / bounded_dist³
        5. `verify_sqrt_bounded` — sqrt safe for all squared distances up to (200 AU)²
    - CI workflow available at `.github/workflows/kani.yml_disabled` (disabled pending CI runner capacity)
- [ ] **Determinism Proof**: Verify that the Symplectic Verlet integrator produces bit-identical results across all supported SIMD instructions (SSE, AVX, NEON).
- [x] **XOF Integrity**: Prove that SHAKE256 output is uniformly distributed across the entire 2048-byte pool when using chaotic inputs. *(Completed 2026-05-11 via 1000-key entropy analysis)*

### 1.2 Cryptographic Hardening
- [x] **Physical Binding**: Include $G$, softening, and force vectors in the hash chain to prevent shortcut attacks. *(Completed 2026-05-11)*
- [x] **Initial Condition Entropy**: Implement $\pm 25\%$ Sun mass randomization to significantly increase the bit-distinct expression space. *(Completed 2026-05-11)*
- [x] **Lyapunov Enforcement**: Programmatically reject configurations that do not reach the required entropy threshold within the requested step count. *(Completed 2026-05-20)*
- [ ] **Stream Authentication**: Integrate NIST SP 800-185 standard KMAC128 or HMAC-SHA256 authenticated tagging into the fast V3 (Photon) and H (Quantum) stream ciphers to defeat ciphertext malleability.

### 1.3 Side-Channel Resistance
- [ ] **Constant-Time Audit**: Use tools like `dudect-bencher` to verify that all secret-dependent code (Phase 1 simulation) is constant-time.
- [/] **Zeroization Verification**: Ensure all secret material is effectively cleared from memory. *(Completed: Zeroize implemented and unit-tested for all critical buffers/states; Pending: assembly audit for compiler optimization removal)*

### 1.4 Statistical Testing
- [x] **NIST SP 800-90B Health Tests**: Implement statistical test suite for keystream quality validation. *(Completed 2026-05-22)*
    - Repetition Test (§4.4.1) — detects consecutive identical bytes
    - Adaptive Proportion Test (§4.4.2) — sliding window byte frequency analysis
    - Runs Test (§2.3) — bit-level run count vs. expected
    - Longest Run Test (§2.4) — longest consecutive identical bits
    - Shannon Entropy calculation (target: >7.5 bits/byte)
    - Chi-square byte distribution test (df=255, critical: 310)
    - Adjacent-byte correlation (Pearson, target: <0.01)
    - Integrated into `tests/entropy_analysis/` with `--keystream` mode

### 1.5 Fuzzing
- [ ] **Continuous Fuzzing**: Implement `cargo-fuzz` (libFuzzer) for:
    - `OrbitalConfig` deserialization (JSON and Binary).
    - The simulation state machine (detecting infinite loops or hangs).
    - `Kelvin` API entry points.
- [ ] **Differential Fuzzing**: Compare the Rust implementation against a high-precision reference (e.g., Python `mpmath`) to detect edge-case divergence.

### 1.6 External Audit
- [ ] **Audit Readiness**: Prepare a "Security Target" document explaining the mathematical foundations and security proofs.
- [ ] **Third-Party Engagement**: Schedule a professional security audit by a specialized firm (e.g., Trail of Bits, NCC Group, or Kudelski Security).

---

## 2. Library Ecosystem (Language Bindings)

Production use cases often require Kelvin to run in non-Rust environments. We will generate high-level libraries ("wrappers") around the core.

### 2.1 Python (`kelvin-py`)
- [ ] **Implementation**: Build a high-level Python package using [PyO3](https://pyo3.rs/) or [cffi](https://cffi.readthedocs.io/).
- [ ] **Distribution**: Publish to PyPI with pre-built wheels for Linux, macOS, and Windows.

### 2.2 JavaScript/TypeScript (`kelvin-js`)
- [ ] **WASM Integration**: Refine the WASM build to ensure `no_std` compatibility.
- [ ] **NPM Package**: Create a package providing a Promise-based API for web and Node.js.
- [ ] **Web Worker Support**: Provide built-in support for running simulation (setup) in background workers to avoid UI blocking.

### 2.3 Mobile (iOS/Android)
- [ ] **Swift Package**: Create a `Kelvin.swift` wrapper around the FFI for seamless iOS integration.
- [ ] **Kotlin/JNI**: Create a `kelvin-android` library with JNI bindings.

---

## 3. Infrastructure & CI/CD

Automate everything to ensure quality and prevent regressions.

### 3.1 Multi-Platform CI
- [ ] **Architecture Support**: Test in CI on `x86_64`, `aarch64` (ARM64), `riscv64`, and `wasm32`.
- [ ] **Endianness Verification**: Explicitly test on big-endian architectures (if possible) to ensure LE-conversion logic is robust.

### 3.2 Supply Chain Security
- [ ] **Dependency Auditing**: Integrate `cargo-audit` and `cargo-deny` into CI.
- [ ] **Secret Scanning**: Use `gitleaks` or similar to prevent accidental leakage of test vectors or keys.

### 3.3 Automated Benchmarking
- [ ] **Regression Detection**: Run `criterion` benchmarks in CI and fail if performance drops by >5% on core simulation paths.

---

## 4. Operational Maturity

### 4.1 Versioning & Stability
- [ ] **SemVer 1.0.0 Roadmap**: Define the stable API surface and commit to no breaking changes for the 1.x lifecycle.
- [ ] **Deprecation Policy**: Establish a clear process for retiring old orbital configuration versions.

### 4.2 Security Policies
- [ ] **SECURITY.md**: Create a policy for vulnerability reporting (Bug Bounty, PGP keys, contact info).
- [ ] **Disclosure Plan**: Define how security advisories will be communicated to users.

### 4.3 Documentation
- [ ] **Kelvin Book**: Expand documentation into a full [mdBook](https://rust-lang.github.io/mdBook/) including:
    - Mathematical Proof of Chaos.
    - Deployment Best Practices.
    - Threat Modeling for specific industries (IoT, Finance).

---

## 5. Execution Timeline

| Phase | Focus | Duration | Status |
| :--- | :--- | :--- | :--- |
| **I: Hardening** | Kani, SP 800-90B, Fuzzing, CT-Audit | 4 Weeks | ✅ Kani & SP 800-90B complete; Fuzzing & CT-Audit pending |
| **II: Ecosystem** | Python & JS Bindings | 3 Weeks | ⬜ Not started |
| **III: Operations** | CI/CD, Security Policies, Docs | 2 Weeks | ⬜ Not started |
| **IV: Audit** | Third-party review & fixes | 4-8 Weeks | ⬜ Not started |

---

> [!IMPORTANT]
> Until Phase IV is complete, all libraries MUST retain the `EXPERIMENTAL` warning in their headers and console output.
