# Quantum Resistance Analysis: Kelvin Cryptosystem

This document evaluates the security of Kelvin in the context of a Post-Quantum (PQ) threat model, specifically considering Grover's and Shor's algorithms.

## 1. Executive Summary

| Component | Primitive | Quantum Status | Effective Security |
|-----------|-----------|----------------|--------------------|
| **Symmetric Cipher** | ChaCha20 | **Quantum-Resistant** | 128-bit (Grover) |
| **Entropy Extractor** | SHA3-512 | **Quantum-Resistant** | 256-bit (Grover) |
| **Asymmetric Identity** | Curve25519 | **Vulnerable** | < 1-bit (Shor) |
| **Chaos Generator** | N-Body Simulation | **Likely PQ-Safe** | Unquantified |

---

## 2. Detailed Analysis

### 2.1 Symmetric Stream (ChaCha20)
Grover's algorithm provides a square-root speedup for unstructured search. For a 256-bit key like the one used in Kelvin's ChaCha20 stream, a quantum computer would require ~2^128 operations to find the key. This is still considered computationally infeasible for the foreseeable future.
- **Verdict**: Kelvin's bulk encryption remains secure against quantum attacks.

### 2.2 Asymmetric Identity (Curve25519)
Shor's algorithm can solve the Elliptic Curve Discrete Logarithm Problem (ECDLP) in polynomial time. A sufficiently powerful quantum computer could derive the private `OrbitalKeyPair` from a captured Public Key.
- **Verdict**: The `identify` feature and any ECDH handshakes are **not** quantum-resistant. They should be used for identity verification only in classical environments.

### 2.3 The Chaos Generator (N-Body KDF)
The core of Kelvin is the high-dimensional chaotic state space of the n-body problem.
- **Classical Complexity**: The n-body problem ($N \ge 3$) has no closed-form solution and is sensitive to the **Butterfly Effect**.
- **Quantum Complexity**: There are no known quantum algorithms that provide an exponential speedup for simulating chaotic classical dynamics. Because the simulation is strictly sequential (Step N depends on Step N-1), it cannot be trivially parallelized or "solved" by quantum superposition.
- **Verdict**: The transition from `OrbitalConfig` to `Seed` is likely a quantum-safe one-way function.

---

## 3. Analysis Tools

To verify the quality of Kelvin's chaos and its resistance to pattern analysis, the following tools are provided:

### 3.1 Avalanche Verification (Sensitivity to Initial Conditions)
This tool verifies that flipping a single bit in the `OrbitalConfig` (e.g., a planet's position by $10^{-19}$ meters) results in a completely different 512-bit seed.

```bash
# Run the internal chaos quality suite
cargo test -p kelvin-kdf --test chaos_test
```

### 3.2 Keystream Entropy Analysis
You can use the CLI to generate a raw keystream and pipe it to standard entropy testing tools like `ent` or `dieharder`.

```bash
# Generate 1MB of raw keystream
kelvin encrypt --config key.json --input /dev/zero --output keystream.raw

# Analyze with 'ent' (if installed)
ent keystream.raw
```

### 3.3 Potential PQ Upgrades (Future Work)
To make Kelvin fully quantum-resistant, the Curve25519 layer must be replaced or augmented with a Post-Quantum Asymmetric primitive, such as:
- **Kyber (ML-KEM)**: Lattice-based key encapsulation.
- **Dilithium (ML-DSA)**: Lattice-based digital signatures.
