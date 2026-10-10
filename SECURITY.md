# Security Policy

> Last updated 2026-10-10. Status tracking lives in [COMPLIANCE.md](COMPLIANCE.md);
> full assumption catalog in [documentation/security_assumptions.md](documentation/security_assumptions.md)
> (7 assumptions); threat model in [THREAT_MODEL.md](THREAT_MODEL.md).
> Full `test_all.bat` gate green 2026-10-10 (release builds, clippy, fmt,
> 215 lib tests, all integration suites, NIST/DudeCT/zeroize).

## Supported Versions

Currently, only the latest version of Kelvin is supported for security updates.

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

**EXPERIMENTAL STATUS**: Kelvin is currently in an experimental phase. While we take security seriously, it should not be used for production data. See [THREAT_MODEL.md](THREAT_MODEL.md) for the current threat model and known limitations.

To report a vulnerability, please open a [GitHub Security Advisory](https://github.com/nliaudat/kelvin/security/advisories/new) or email the maintainer directly. We aim to acknowledge receipt within 48 hours and provide a fix timeline within 5 business days.

## Security Assumptions

The security of the Kelvin cryptosystem depends on the following assumptions. Users should understand these before deploying Kelvin in any capacity.

### Assumption 1: SHAKE256 is a secure PRF

The keystream is derived from the orbital state via SHAKE256 (a NIST-standardized XOF). We assume SHAKE256 behaves as a secure pseudorandom function — i.e., its output is computationally indistinguishable from random for any adversary without knowledge of the input state.

- **Standard**: NIST SP 800-185 / FIPS 202
- **Status**: NIST-approved, widely deployed in post-quantum cryptography (Kyber, Dilithium)
- **Risk**: A cryptanalytic break of SHAKE256 would compromise all modes (V1, V2, V3, H)

### Assumption 2: Orbital chaos feeds the extractor (no quantified min-entropy rate)

The n-body simulation amplifies small differences in initial conditions
into exponential divergence (Lyapunov chaos), and the extractor hashes the
resulting state via SHAKE256. No proven bits-per-reseed figure is claimed:
the old "≥256 bits of min-entropy per reseed" statement is withdrawn —
C1's rate chain is under re-derivation and the empirical figure measures
local sensitivity, not a rate (see C1/gap1 2026-10-10 notes). What holds:
configurations below the measured chaos horizon are rejected at init.

- **Validation**: Lyapunov time estimation rejects configurations below the entropy threshold
- **Testing**: NIST SP 800-90B health tests validate keystream quality
- **Risk**: A configuration with insufficient chaos (e.g., too few bodies, too few steps) may produce predictable keystream. This is mitigated by Lyapunov enforcement at initialization.

### Assumption 3: BLAKE3 reseed provides key-derivation chaining (not PFS)

When the key schedule reseeds, the new key is derived from the original seed via BLAKE3. We assume BLAKE3 is a secure PRF, ensuring that compromise of a single session key does not reveal past or future keys *through the schedule*. This is **not** Perfect Forward Secrecy: anyone holding the orbital config can recompute all keys from scratch.

- **Standard**: BLAKE3 (O'Connor, Aumasson, Neves, & Wilcox-O'Hearn, 2021)
- **Status**: Fast, well-analyzed hash function
- **Risk**: A cryptanalytic break of BLAKE3 would compromise schedule chaining guarantees

### Assumption 4: Integer-only arithmetic bounds timing hazards (not a constant-time proof)

The Q32.64 simulation uses integer arithmetic (no floating point in the
core loop), removing the largest cross-platform timing hazard. This is
**not** a constant-time guarantee: the code contains operand-dependent
branches (e.g. sign handling, zero-divisor checks), and no full-pipeline
timing measurement exists.

- **Validation**: dudect-bencher t-test passes (|t| < 5) on 12 primitive-level benches (arithmetic, extract, simulate, schedule — gated in `test_all.bat`)
- **Risk**: A timing leak in the simulation could reveal orbital state information. The `compute_accelerations` function passes independently; `verlet_step` timing variation is under investigation (see [THREAT_MODEL.md](THREAT_MODEL.md)).

## Limitations

### Authentication (on by default for XOR modes)

V1 (ChaCha20Poly1305) is AEAD. V2/V3/H append a KMAC128 tag by default
(`--no-auth` opts out) — a quantum adversary gains only Grover's speedup
(2^128) to forge tags. Running unauthenticated XOR modes accepts
malleability; that is an explicit opt-out, not the default.

### Session discipline (config, nonce)

Security requires one (config, nonce) pair per instance: the CLI
generates a fresh nonce at encrypt time and requires it at decrypt time
(see [COMPLIANCE.md](COMPLIANCE.md) §3). Same-nonce reuse reuses
keystream (two-time pad).

### No key encapsulation (KEM)

Kelvin is a symmetric-only cryptosystem. It does not provide public-key encryption or key exchange. To establish a shared secret between two parties, you must use an external KEM (e.g., ML-KEM/FIPS 203, X25519).

- **Mitigation**: ML-KEM-768 / ML-DSA-65 primitives are already integrated via `kelvin-kdf` (`ml-kem`, `ml-dsa` dependencies) for identity flows; there is no separate KEM wrapper around the symmetric modes.
- **Alternative**: `OrbitalKeyPair` derives X25519/Ed25519 key material from orbital state for key derivation use cases — it is not an identity-based key-agreement protocol.

## Disclosure Plan

Security advisories will be published via:

1. GitHub Security Advisories (https://github.com/nliaudat/kelvin/security/advisories)
2. Release notes for patched versions
3. Direct notification to registered downstream package maintainers

We follow a 90-day disclosure timeline: 90 days after a fix is released, we publish full details of the vulnerability.
