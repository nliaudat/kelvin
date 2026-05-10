# Quantum Resistance Analysis: Kelvin Cryptosystem

This document evaluates the security of Kelvin in the context of a Post-Quantum (PQ) threat model, specifically considering Grover's and Shor's algorithms.

## 1. Executive Summary

| Component | Primitive | Quantum Status | Effective Security |
|-----------|-----------|----------------|--------------------|
| **Symmetric Cipher** | ChaCha20 | **Quantum-Resistant** | 128-bit (Grover) |
| **Entropy Extractor** | SHA3-512 | **Quantum-Resistant** | 256-bit (Grover) |
| **Asymmetric Identity** | **ML-DSA-65** | **Quantum-Resistant** | NIST Level 3 (Lattice) |
| **Chaos Generator** | N-Body Simulation | **Likely PQ-Safe** | Unquantified |

---

## 2. Detailed Analysis

### 2.1 Symmetric Stream (ChaCha20)
Grover's algorithm provides a square-root speedup for unstructured search. For a 256-bit key like the one used in Kelvin's ChaCha20 stream, a quantum computer would require ~2^128 operations to find the key. This is still considered computationally infeasible for the foreseeable future.
- **Verdict**: Kelvin's bulk encryption remains secure against quantum attacks.

### 2.2 Asymmetric Identity (Hybrid PQC)
Kelvin uses a hybrid asymmetric layer that defaults to **ML-DSA-65** (FIPS 204) for identity verification.
- **ML-DSA-65**: Based on the Module Learning with Errors (M-LWE) problem, it is designed to be resistant to Shor's algorithm and is standardized for post-quantum signatures.
- **ML-KEM-768**: Also supported for key encapsulation (FIPS 203), providing a quantum-safe transition for shared secrets.
- **Curve25519 (Legacy)**: Remains available for classical compatibility but is vulnerable to Shor's algorithm.

---

### 2.3 The Chaos Generator (N-Body KDF)
The core of Kelvin is the high-dimensional chaotic state space of the n-body problem.
- **Classical Complexity**: The n-body problem ($N \ge 3$) has no closed-form solution and is sensitive to the **Butterfly Effect**.
- **Quantum Complexity**: There are no known quantum algorithms that provide an exponential speedup for simulating chaotic classical dynamics. Because the simulation is strictly sequential (Step N depends on Step N-1), it cannot be trivially parallelized or "solved" by quantum superposition.
- **Verdict**: The transition from `OrbitalConfig` to `Seed` is likely a quantum-safe one-way function.

---

## 3. Analysis Tools

### 3.1 CLI Identify (Public Key Inspection)
The `identify` command allows you to inspect your PQ and classical identities:

```bash
# Show default PQ-Signature (ML-DSA-65)
kelvin identify --config key.json

# Show all identities (including Curve25519 and ML-KEM)
kelvin identify --config key.json --all
```

### 3.2 Avalanche Verification
Flipping a single bit in the `OrbitalConfig` results in a completely different set of PQ-keys. This can be verified with:

```bash
# Run the internal chaos quality suite
cargo test -p kelvin-kdf --test chaos_test
```

