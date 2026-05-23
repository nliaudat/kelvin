# Security Policy

## Supported Versions

Currently, only the latest version of Kelvin is supported for security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**EXPERIMENTAL STATUS**: Kelvin is currently in an experimental phase. While we take security seriously, it should not be used for production data until the "Production Readiness Plan" is fully executed.

To report a vulnerability, please open a [GitHub Security Advisory](https://github.com/nliaudat/kelvin/security/advisories/new) or email the maintainer directly. We aim to acknowledge receipt within 48 hours and provide a fix timeline within 5 business days.

## Security Assumptions

The security of the Kelvin cryptosystem depends on the following assumptions. Users should understand these before deploying Kelvin in any capacity.

### Assumption 1: SHAKE256 is a secure PRF

The keystream is derived from the orbital state via SHAKE256 (a NIST-standardized XOF). We assume SHAKE256 behaves as a secure pseudorandom function — i.e., its output is computationally indistinguishable from random for any adversary without knowledge of the input state.

- **Standard**: NIST SP 800-185 / FIPS 202
- **Status**: NIST-approved, widely deployed in post-quantum cryptography (Kyber, Dilithium)
- **Risk**: A cryptanalytic break of SHAKE256 would compromise all modes (V1, V2, V3, H)

### Assumption 2: Orbital chaos provides ≥256 bits of min-entropy per reseed

The n-body gravitational simulation amplifies small differences in initial conditions into exponential divergence (Lyapunov chaos). We estimate that the orbital state provides at least 256 bits of min-entropy per reseed interval.

- **Validation**: Lyapunov time estimation rejects configurations below the entropy threshold
- **Testing**: NIST SP 800-90B health tests validate keystream quality
- **Risk**: A configuration with insufficient chaos (e.g., too few bodies, too few steps) may produce predictable keystream. This is mitigated by Lyapunov enforcement at initialization.

### Assumption 3: BLAKE3 reseed provides forward secrecy

When the key schedule reseeds, the new key is derived from the original seed via BLAKE3. We assume BLAKE3 is a secure PRF, ensuring that compromise of a single session key does not reveal past or future keys.

- **Standard**: BLAKE3 (Baish et al., 2020)
- **Status**: Fast, well-analyzed hash function
- **Risk**: A cryptanalytic break of BLAKE3 would compromise forward secrecy guarantees

### Assumption 4: Fixed-point arithmetic is constant-time

The Q32.64 fixed-point arithmetic used in the n-body simulation is implemented without secret-dependent branches or variable-time operations.

- **Validation**: dudect-bencher t-test passes (|t| < 5) for all individual arithmetic operations
- **Risk**: A timing leak in the simulation could reveal orbital state information. The `compute_accelerations` function passes independently; the `verlet_step` timing variation is a benchmark artifact (see production readiness plan §1.3).

## Limitations

### No post-quantum authentication (without optional module)

The standard authenticated modes (V1 AEAD, V3/H with BLAKE3-keyed MAC) use classical cryptography for authentication. They are **not** post-quantum secure for authentication purposes. An adversary with a quantum computer could forge MAC tags.

- **Mitigation**: The optional HAWK-512 module (Phase V) provides post-quantum digital signatures
- **Timeline**: Planned for future release, dependent on NIST PQC standardization

### No key encapsulation (KEM)

Kelvin is a symmetric-only cryptosystem. It does not provide public-key encryption or key exchange. To establish a shared secret between two parties, you must use an external KEM (e.g., ML-KEM/FIPS 203, X25519).

- **Mitigation**: A hybrid PQ + OTP KEM wrapper is planned for Phase V
- **Alternative**: Use the `OrbitalKeyPair` asymmetric key derivation for identity-based key agreement

## Disclosure Plan

Security advisories will be published via:

1. GitHub Security Advisories (https://github.com/nliaudat/kelvin/security/advisories)
2. Release notes for patched versions
3. Direct notification to registered downstream package maintainers

We follow a 90-day disclosure timeline: 90 days after a fix is released, we publish full details of the vulnerability.
