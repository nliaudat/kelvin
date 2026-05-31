# Threat Model

## Overview

This document describes the attacker capabilities, security boundaries, and trust assumptions for the Kelvin cryptosystem. It is intended for security reviewers, auditors, and developers integrating Kelvin into their systems.

---

## 1. Attacker Model

### 1.1 Dolev-Yao Attacker (Network)

Kelvin assumes a standard Dolev-Yao network attacker:

- **Eavesdropping**: The attacker can read all ciphertext, public keys, and metadata transmitted between parties.
- **Tampering**: The attacker can modify, reorder, replay, or drop messages in transit.
- **Injection**: The attacker can inject arbitrary messages into the communication channel.
- **No side channels**: The attacker cannot observe internal state (memory, timing, CPU) of the communicating parties over the network.

### 1.2 Local Attacker (Side Channels)

For local attacks, we consider:

- **Timing attacker**: Can measure execution time of cryptographic operations (mitigated by constant-time benchmarks and dudect auditing).
- **Memory attacker**: Can read process memory after secret material is used (mitigated by `zeroize` on all secret structs).
- **Cache attacker**: Can observe cache-timing patterns (mitigated by constant-time XOR operations and KMAC128 verification via `subtle::ConstantTimeEq`).

### 1.3 Quantum Attacker

Kelvin provides:

- **128-bit post-quantum security** for SHAKE256-based operations (Grover's algorithm square-root speedup).
- **No quantum key agreement**: Kelvin does not implement QKD or other quantum-channel protocols.
- **Hybrid identity**: ML-DSA-65 (FIPS 204) and ML-KEM-768 (FIPS 203) provide NIST-standardized post-quantum signatures and KEM.

---

## 2. Security Boundaries

### 2.1 What Kelvin Protects

| Asset | Protection | Mechanism |
|-------|-----------|-----------|
| Orbital configuration (shared secret) | Confidentiality, integrity | Never transmitted; must be pre-shared out-of-band |
| Encryption keystream | Confidentiality | SHAKE256 XOF with domain separation |
| Ciphertext integrity (authenticated modes) | Integrity, authenticity | KMAC128 (NIST SP 800-185) with constant-time verification |
| Post-quantum identity keys | Confidentiality, authenticity | ML-DSA-65 / ML-KEM-768 / Curve25519 derived from orbital state |

### 2.2 What Kelvin Does NOT Protect

- **Metadata**: Message length, timing, and communication patterns are not hidden.
- **Forward secrecy for long-lived keys**: If the orbital configuration is compromised, all past and future keystreams derived from it are compromised. Per-message reseeding mitigates this within a session.
- **Denial of service**: The n-body simulation is computationally expensive. An attacker can force a legitimate party to waste CPU time by initiating many connections.
- **Side-channel resistance of the simulation loop**: The Verlet/Euler integrator is not constant-time. Timing variations of |t| ≈ 75 have been measured in dudect benchmarks. Individual `Fixed` operations and `compute_accelerations` pass independently (|t| < 5), but the composite integrator shows measurable timing variation.
  - **For V3/H/Prism/Split/Flare modes**: The simulation runs once before any keystream is produced. Timing variations during this one-time setup do not leak keystream material in a practical sense, as no secret data is being input or output during the simulation.
  - **⚠️ For V2 (Chaos) mode**: This defense does NOT apply. V2 interleaves simulation with keystream generation — the simulation advances one step per chunk of data processed. Timing variations in the simulation loop correlate with orbital state and may be observable by an attacker in streaming scenarios. Additionally, if any intermediate orbital state is compromised (via timing side-channel, memory disclosure, or checkpointing), an attacker can forward-simulate from that point to decrypt all subsequent traffic.

### 2.3 Trust Assumptions

1. **Pre-shared configuration**: Communicating parties must share the `OrbitalConfig` out-of-band. There is no key exchange protocol.
2. **Same integration method**: Both parties must use the same integrator (Verlet or Euler) and the same fixed-point arithmetic. This is guaranteed by the library.
3. **Deterministic execution**: The simulation must be bit-identical across platforms. This is verified by golden hash tests in `tests/kelvin_tests/determinism.rs`.
4. **No malicious hardware**: Kelvin does not protect against hardware-level attacks (JTAG, voltage glitching, etc.).

---

## 3. Cryptographic Assumptions

### 3.1 N-Body Chaos as a One-Way Function

The security of Kelvin's KDF rests on the computational difficulty of inverting an n-body gravitational simulation:

- **Forward direction**: Given initial conditions, simulate N steps → trivial (O(N × n²) fixed-point operations).
- **Reverse direction**: Given final state, recover initial conditions → computationally infeasible for N ≥ 3 due to:
  - Exponential divergence of nearby trajectories (Lyapunov exponent)
  - Numerical dissipation in Euler integration (one-way information loss)
  - SHAKE256 hashing of the final state (preimage resistance)

### 3.2 Entropy Source

Kelvin does **not** rely on system entropy (OS RNG) for keystream generation. All entropy is derived from the chaotic n-body dynamics:

- **Initial entropy**: The orbital configuration (positions, velocities, masses, G, softening, steps) is the shared secret.
- **Chaotic amplification**: The Lyapunov exponent amplifies microscopic differences exponentially.
- **Entropy extraction**: SHAKE256 XOF extracts uniform random bits from the final orbital state.

### 3.3 Security Levels

| Level | Bodies | Steps | Raw Keyspace | Equivalent Security |
|-------|--------|-------|-------------|-------------------|
| Standard | 5 | 1,000,000 | ~2¹²⁸⁷ | 128-bit (SHAKE256 bound) |
| Paranoid | 5 | 10,000,000 | ~2¹²⁸⁷ | 128-bit (SHAKE256 bound) |
| Maximum | 10 | 100,000,000 | ~2²⁷⁴⁴ | 128-bit (SHAKE256 bound) |

The raw keyspace exceeds the security level of the underlying hash function (SHAKE256 provides 256-bit classical / 128-bit quantum security). The practical security is bounded by the hash function, not the keyspace. **All three levels provide identical effective security** — the extra steps in Paranoid and Maximum increase simulation setup cost but do not raise the security bound. Post-quantum effective security: 128-bit (Grover's bound on SHAKE256).

---

## 4. Comparison with Existing KDFs

| Property | Kelvin | Argon2id | scrypt | HKDF |
|----------|--------|----------|--------|------|
| **Hardness type** | Computational (n-body sim) | Memory-hard | Memory-hard | Lightweight |
| **Side-channel resistance** | Partial (sim not constant-time) | Good | Good | Excellent |
| **Quantum resistance** | 128-bit (SHAKE256) | Partial (BLAKE2b) | Limited (SHA-256 via PBKDF2) | Limited (SHA-512) |
| **Deterministic** | ✅ Yes | ✅ Yes | ✅ Yes | ✅ Yes |
| **Platform-independent** | ✅ Yes (fixed-point) | ❌ No (memory layout varies) | ❌ No (memory layout varies) | ✅ Yes |
| **Setup cost** | Seconds–minutes | Configurable | Configurable | Microseconds |
| **Throughput** | ~200–500 MB/s | ~1 GB/s | ~500 MB/s | ~10 GB/s |
| **Key stretching** | Implicit (simulation cost) | Explicit (iterations) | Explicit (iterations) | None |

---

## 5. Attack Scenarios

### 5.1 Brute-Force Configuration

**Attacker goal**: Recover the orbital configuration from ciphertext.

**Difficulty**: The configuration space is bounded by the raw keyspace (~2¹²⁸⁷ for Standard). Each guess requires running the full n-body simulation, which is computationally expensive. An attacker with 10⁶ cores would need ~10³⁷⁰ years to exhaust the Standard keyspace.

### 5.2 Shortcut Attack

**Attacker goal**: Derive the keystream without running the simulation.

**Difficulty**: The SHAKE256 seed extraction is a one-way function. The attacker would need to either:
- Invert SHAKE256 (preimage resistance: 2²⁵⁶ classical, 2¹²⁸ quantum)
- Find a collision in the orbital state space (birthday bound: 2¹²⁸)
- Exploit a weakness in the fixed-point arithmetic (none known)

### 5.3 Timing Side Channel

**Attacker goal**: Extract the orbital configuration from timing measurements.

**Difficulty**: For V3/H/Prism/Split/Flare modes, the simulation runs once before any keystream is produced, so timing variations do not leak keystream material. The XOR encryption/decryption is constant-time (verified by dudect benchmarks).

**⚠️ For V2 (Chaos) mode**: The simulation is interleaved with keystream generation, making per-step timing variations a potential concern. This is an acknowledged limitation requiring further investigation.

### 5.4 Quantum Attack

**Attacker goal**: Use a quantum computer to break Kelvin.

**Difficulty**: SHAKE256 provides 128-bit post-quantum security (Grover's algorithm). The n-body simulation itself is not quantum-accelerable — there is no known quantum algorithm for simulating chaotic n-body systems faster than classical methods.

---

## 6. Security Recommendations for Deployments

1. **Use authenticated modes** (`KelvinPhotonAuthenticated` or `KelvinQuantumAuthenticated`) for all data in transit. The unauthenticated modes (V3 Photon, H Quantum) are malleable.
2. **Pre-share configurations securely** — the orbital configuration is the equivalent of a cryptographic key. Use a secure channel (e.g., Signal, PGP-encrypted email) to exchange it.
3. **Store configurations encrypted at rest** — use OS keychain facilities or hardware security modules to protect `key.json` files on disk.
4. **Set appropriate security levels** — Standard is sufficient for most use cases. Paranoid and Maximum provide additional simulation cost for higher assurance.
5. **Monitor for configuration reuse** — if the same orbital configuration is used for multiple messages, an attacker who learns one keystream can decrypt all messages. Use per-message reseeding or unique configurations per session.
6. **Keep the library updated** — security fixes and improvements will be documented in release notes.

---

## References

- Dolev, D., & Yao, A. C. (1983). "On the security of public key protocols." *IEEE Transactions on Information Theory*, 29(2), 198–208.
- Krawczyk, H., & Eronen, P. (2010). "HMAC-based Extract-and-Expand Key Derivation Function (HKDF)." RFC 5869.
- NIST FIPS PUB 202 (2015). "SHA-3 Standard: Permutation-Based Hash and Extendable-Output Functions."
- NIST FIPS PUB 204 (2024). "Module-Lattice-Based Digital Signature Standard."
- NIST FIPS PUB 203 (2024). "Module-Lattice-Based Key-Encapsulation Mechanism Standard."
- Bernstein, D. J., & Lange, T. (2017). "Post-quantum cryptography." *Nature*, 549, 188–194.