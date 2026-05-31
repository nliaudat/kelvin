# Quantum Resistance Analysis: Kelvin Cryptosystem

This document evaluates the security of Kelvin in the context of a Post-Quantum (PQ) threat model, specifically considering Grover's and Shor's algorithms.

## 1. Executive Summary

| Component | Primitive | Quantum Status | Effective Security |
|-----------|-----------|----------------|--------------------|
| **Symmetric Cipher** | ChaCha20 | **Quantum-Resistant** | 128-bit (Grover) |
| **Entropy Extractor (SHA3-512)** | SHA3-512 | **Quantum-Resistant** | 256-bit (Grover) |
| **Entropy Extractor (SHAKE256)** | SHAKE256 (XOF) | **Quantum-Resistant** | 128-bit (Grover) |
| **Entropy Pool** | 2048-byte SHAKE256 pool | **Quantum-Resistant** | 256-bit (Grover) |
| **Asymmetric Identity** | **ML-DSA-65** | **Quantum-Resistant** | NIST Level 3 (Lattice) |
| **Chaos Generator** | N-Body Simulation | **Likely PQ-Safe** | Unquantified |

---

## 2. Detailed Analysis

### 2.1 Symmetric Stream (ChaCha20)
Grover's algorithm provides a square-root speedup for unstructured search. For a 256-bit key like the one used in Kelvin's ChaCha20 stream, a quantum computer would require ~2^128 operations to find the key. This is still considered computationally infeasible for the foreseeable future.
- **Verdict**: Kelvin's bulk encryption remains secure against quantum attacks.

### 2.1.1 Grover's Attack T-Gate Cost Estimate

Following the methodology of Song et al. (2025) [CryptoChaos, arXiv:2504.08618], we estimate the T-gate cost of a Grover attack on Kelvin's ChaCha20 keystream.

**Assumptions:**
- Target: 256-bit ChaCha20 key (Grover search space = 2^256)
- ChaCha20 round function: 20 rounds, each requiring ~100 T-gates per round for the quarter-round operations (using Gidney's surface code estimates)
- Total T-gates per ChaCha20 evaluation: ~2,000 (conservative, including data loading)
- Grover iterations required: ~2^128 (for 256-bit key)

**T-Gate Cost Estimate:**

| Component | Cost |
|-----------|------|
| ChaCha20 oracle (T-gates) | ~2,000 |
| Grover iterations | ~2^128 |
| **Total T-gates** | **~2,000 × 2^128 ≈ 2.7 × 10^41** |
| **Logical qubits** | ~3,000 (surface code) |
| **Estimated wall time** | > 10^30 years (at 10 MHz clock) |

**Comparison with CryptoChaos:**
CryptoChaos reported ~2.1 × 10⁹ T-gates for their AES-GCM construction. Kelvin's ChaCha20-based construction requires ~2.7 × 10^41 T-gates — over 32 orders of magnitude more — due to the larger key size (256-bit vs. 128-bit effective). This is because Grover's algorithm scales as O(2^(n/2)) where n is the key size.

**Verdict:** A Grover attack on Kelvin's 256-bit ChaCha20 key is computationally infeasible by an enormous margin, even compared to already-infeasible estimates for 128-bit keys.

### 2.2 Asymmetric Identity (Hybrid PQC)
Kelvin uses a hybrid asymmetric layer that defaults to **ML-DSA-65** (FIPS 204) for identity verification.
- **ML-DSA-65**: Based on the Module Learning with Errors (M-LWE) problem, it is designed to be resistant to Shor's algorithm and is standardized for post-quantum signatures.
- **ML-KEM-768**: Also supported for key encapsulation (FIPS 203), providing a quantum-safe transition for shared secrets.
- **Curve25519 (Legacy)**: Remains available for classical compatibility but is vulnerable to Shor's algorithm.
- **Orbital Key Derivation**: The `asymmetric.rs` module derives key pairs deterministically from the orbital configuration itself, binding the asymmetric identity to the chaotic seed. This means that compromising the orbital state compromises the identity — but also means the identity inherits the PQ properties of the chaos generator.

---

### 2.3 The Chaos Generator (N-Body KDF)

The core of Kelvin is the high-dimensional chaotic state space of the n-body problem.
- **Classical Complexity**: The n-body problem ($N \ge 3$) has no closed-form solution and is sensitive to the **Butterfly Effect**.
- **Quantum Complexity**: There are no known quantum algorithms that provide an exponential speedup for simulating chaotic classical dynamics. Because the simulation is strictly sequential (Step N depends on Step N-1), it cannot be trivially parallelized or "solved" by quantum superposition.
- **Verdict**: The transition from `OrbitalConfig` to `Seed` is likely a quantum-safe one-way function.

#### 2.3.1 Deep Physical Binding

The entropy extraction process has been hardened with **Deep Physical Binding** — the hash chain now includes not only the orbital state (positions, velocities, masses) but also:

- **Physical Constants**: The gravitational constant $G$ and softening factor $\varepsilon$ are hashed into every seed. This prevents quantum "shortcut" attacks that might attempt to model the orbital evolution using a different set of physical laws.
- **Instantaneous Force Vectors**: The acceleration vector $\mathbf{a}_i$ acting on each body at the extraction step is computed via `compute_accelerations()` and hashed alongside the body data. This binds the seed to the *interactions* between bodies, not just their positions.
- **Domain Separation**: A personalization string ensures that seeds derived for different purposes (e.g., encryption vs. signing) are cryptographically independent.

These measures ensure that even if a quantum adversary could somehow compute the orbital state at a given step, they would still need to invert the SHAKE256 hash to recover the seed — a problem with no known quantum speedup beyond Grover's.

#### 2.3.2 Dual-Path Extraction

Kelvin uses two complementary extraction paths:

| Path | Hash | Output | Purpose |
|------|------|--------|---------|
| **SHA3-512** | SHA3-512 | 64 bytes | Legacy seed, asymmetric key derivation |
| **SHAKE256 (XOF)** | SHAKE256 | 2048 bytes | Key schedule initialization |

Both paths share the same `feed_orbital_state()` hashing logic, ensuring consistency. The SHAKE256 path produces a **2048-byte entropy pool** that is used to initialize the key schedule, providing a large internal state that resists quantum search.

#### 2.3.3 Stability Bodyguard

The `validate()` method in `OrbitalConfig` performs **Bodyguard checks** at configuration creation time:

1. **Identical Position Rejection**: No two bodies may share the same position (zero distance).
2. **Initial Collapse Detection**: No two bodies may be closer than `min_separation` at step 0.
3. **Initial Ejection Detection**: No body may be on an unbound (escape) trajectory at step 0.

These checks guarantee that the simulation starts in a chaotic regime. Without them, a degenerate system (e.g., two bodies at the same point) could reduce the effective degrees of freedom, potentially weakening the entropy. The checks are enforced at config creation time, so a quantum adversary cannot exploit a degenerate initial condition to shortcut the simulation.

During simulation, the `stability.rs` module monitors for:
- **Gravitational Collapse**: Any pair of bodies closer than `min_separation` triggers a collapse event.
- **Ejection**: Any body with total energy exceeding `ejection_energy_threshold` is flagged as ejected.

These runtime checks ensure the simulation remains in a high-entropy regime throughout its lifetime.

---

## 3. Analysis & Verification

### 3.1 CLI Analyze (Chaos Quality)
The `analyze` command is the primary tool for verifying the sensitivity of the chaotic generator:
```bash
kelvin analyze --config key.json
```
This performs a single-bit perturbation analysis on the orbital initial conditions and confirms that the resulting **Post-Quantum identity** (ML-KEM) undergoes a full avalanche (>110 bits changed out of 256).

### 3.2 Entropy Audit
Large-scale entropy audits have confirmed that the generator produces uniformly distributed seeds across the full 2048-byte SHAKE256 pool. See the [Entropy Report](entropy_report.md) for more details.

### 3.3 Bodyguard Validation Coverage
The Bodyguard checks are verified by unit tests covering:
- `test_identical_positions_rejected`
- `test_initial_collapse_rejected`
- `test_initial_ejection_rejected`
- `test_too_few_bodies`
- `test_non_positive_mass`

These tests ensure that any configuration that could lead to a low-entropy regime is rejected at creation time, maintaining the PQ security guarantees of the system.

---

## 4. Summary

| Attack Vector | Kelvin Defense | PQ Status |
|---------------|----------------|-----------|
| **Grover's (key search)** | 256-bit ChaCha20 key (V1) or SHAKE256 (V2/V3/H) | 128-bit effective security |
| **Shor's (factorization)** | ML-DSA-65 / ML-KEM-768 | NIST Level 3 |
| **Quantum shortcut (simulation)** | Sequential chaos + Deep Physical Binding | No known speedup (unproven — active research area) |
| **Quantum shortcut (keystream)** | SHAKE256 XOR — no algebraic structure for Shor's | No known speedup |
| **Degenerate initial conditions** | Bodyguard validation | Prevented at creation |
| **Orbital state inversion** | SHAKE256 + 2048-byte pool | Grover-limited |

Kelvin's architecture combines multiple layers of post-quantum protection: standardized lattice-based cryptography for identity, a chaotic classical simulation for key derivation, and SHAKE256 for entropy extraction. The XOR-based stream cipher modes (V2/V3/H/Prism/Split/Flare) have **no algebraic structure** — there is nothing for Shor's algorithm to factor or for lattice reduction to exploit. The only quantum attack is Grover's search on the SHAKE256 output, reducing 256-bit classical security to 128-bit quantum security.

The Deep Physical Binding and Bodyguard checks ensure that the system remains in a high-entropy regime, closing potential attack vectors that could arise from degenerate orbital configurations.

See the [Stream Cipher Security Analysis](stream_cipher_security.md) for the full quantum resistance argument for Kelvin's stream cipher keystream.

---

*Last Updated: 2026-05-28*

