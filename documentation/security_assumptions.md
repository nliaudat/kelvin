# Security Assumptions — Kelvin Cryptosystem

> **Document Status:** Living document (corrected as needed; see git history)
> **Last Updated:** 2026-10-10

This document codifies the formal security assumptions underlying the Kelvin
cryptosystem. Each assumption includes:
- A formal statement of the assumption
- The code that enforces or relies on it
- The consequence if the assumption is violated
- Evidence supporting the assumption

---

## Assumption 1: N-Body One-Way Function

> ⚠️ **Status: CONJECTURE — NOT a formal security assumption.** This has NOT been formally reduced to any known hard problem (lattice, discrete log, factoring, or similar). Unlike standard cryptographic assumptions, there is no proof that recovering initial conditions from the orbital state is computationally hard. This is a conjecture based on physical reasoning about chaotic dynamics — plausible but mathematically unproven.

### Statement

Given the final orbital state after `S` simulation steps, it is computationally
infeasible to recover the initial orbital configuration (masses, positions,
velocities) within useful precision.

### Rationale

The N-body problem (N ≥ 3) has no closed-form analytical solution (Poincaré,
1899). Numerical integration is the only path forward. There is no known
algorithm to invert the Verlet or Euler integrator for a general N-body
configuration — an attacker would need to simulate forward from every possible
initial condition to find one matching the final state. However, Poincaré's
theorem only proves there is no analytic closed-form solution — it does NOT
prove that recovering initial conditions from observed output is
computationally hard. That is a separate question.

### Enforcement

- **Lyapunov exponent estimation** (`kelvin-kdf/src/lyapunov.rs`): The shadow
  orbit method estimates the Lyapunov exponent λ. Configurations with
  `total_steps < min_chaos_steps` are rejected at initialization:
  ```rust
  if config.total_steps < result.min_chaos_steps {
      return Err(KelvinError::InsufficientChaos { ... });
  }
  ```
- **Stability monitoring** (`kelvin-core/src/stability.rs`): Ejection and
  collapse detectors ensure N ≥ chaotic body count remains ≥ 3 throughout
  the simulation.

### Violation Consequence

If an attacker could efficiently invert the simulation, they could recover
the shared OrbitalConfig from intercepted ciphertext, breaking the stream cipher.
However, the attacker must also invert SHAKE256 (256-bit preimage resistance),
which bounds the system's security regardless of the n-body layer.
With known plaintext, the keystream is directly revealed (standard stream cipher
property), and the attacker bypasses the n-body simulation entirely to attack
SHAKE256 directly.

### Supporting Evidence

- **Kani proofs L0-L2** (`proofs/kani/`): Formal verification that
  `compute_accelerations` satisfies Newton's laws (action-reaction, direction,
  proportionality to mass).
- **Determinism tests L4** (`tests/kelvin_tests/determinism.rs`): 18 tests
  verify bit-identical simulation results, confirming the forward direction
  is well-defined.
- **Reference**: Kwong et al. (2025), Song et al. (2025), Jawad (2025) —
  independent academic research relying on the same N-body one-way property.

---

## Assumption 2: Fixed-Point Determinism

### Statement

Q32.64 fixed-point arithmetic produces identical simulation results on any
platform executing the same integer operations (by design — no floating
point in the core loop). Multi-architecture coverage (x86_64, aarch64,
riscv64, wasm32) is asserted by integer-only design and single-toolchain
golden-hash tests, not by a cross-platform CI matrix.

### Rationale

Fixed-point arithmetic uses only integer operations (addition, subtraction,
multiplication, division, bit shifts) which are deterministic across all
IEEE-compliant platforms. This contrasts with floating-point arithmetic,
where different platforms may have different rounding behavior, FMA
implementations, or NaN propagation rules.

### Enforcement

- **Q32.64 format** (`kelvin-core/src/fixed_math.rs`): All simulation math
  uses `i128`-based fixed-point with `1.0 = 2^64`. No `f32`/`f64` in the
  core loop (f64 appears only outside the deterministic path: keygen
  spherical sampling, display helpers, estimator statistics).
- **Determinism tests** (18 tests in `tests/kelvin_tests/determinism.rs`):
  golden hashes + intra-process repeatability pass on the tested
  toolchain. SSE2/AVX/AVX2/NEON matrix coverage is asserted by design
  (integer-only arithmetic), not automated — the "pass identically on
  SSE2, AVX, AVX2" phrasing in earlier revisions overstated this.
- **Architecture-agnostic golden hashes**: Golden hashes are captured on
  x86_64; bit-identity elsewhere follows from integer-only arithmetic by
  design (not measured per-architecture).

### Violation Consequence

If different platforms produce different keystreams from the same OrbitalConfig,
communicating parties using different hardware would be unable to decrypt each
other's messages.

### Supporting Evidence

- **Kani harness `verify_mul_no_overflow`**: Proves commutativity (`a*b == b*a`),
  identity (`a*1 == a`), and zero (`a*0 == 0`) for all bounded inputs.
- **18/18 determinism tests pass** on the tested toolchain (see scope note
  above re: SIMD matrix).
- **Reference**: Goldberg (1991) — "What Every Computer Scientist Should Know
  About Floating-Point Arithmetic."

---

## Assumption 3: SHAKE256 Security

### Statement

SHAKE256 (NIST FIPS 202) is a cryptographically secure extendable-output
function (XOF) with:
- **256-bit classical security** against preimage attacks (collision bound
  is output-length dependent: 2^{L/2} for L-bit output)
- **128-bit quantum security** (Grover's algorithm reduces effective security
  by at most half)
- Computational indistinguishability from random for any polynomial-time
  adversary

### Rationale

SHAKE256 is a NIST-standardized hash function based on the Keccak sponge
construction. It is part of the SHA-3 family and has undergone extensive
public cryptanalysis since 2008. No practical preimage, collision, or
distinguishing attacks exist against SHAKE256 at the time of writing.

### Enforcement

- **Entropy extraction** (`kelvin-kdf/src/extractor.rs`): All orbital state
  extraction uses SHAKE256 via `extract_shake256_into()`.
- **Keystream generation** (`kelvin/src/photon.rs`, `kelvin/src/quantum.rs`,
  `kelvin/src/prism.rs`, `kelvin/src/split.rs`, `kelvin/src/flare.rs`): All
  stream cipher modes use SHAKE256 XOR as the core cipher construction.
- **Domain separation**: Each mode and operation uses a distinct domain
  separator string (e.g., `DOMSEP_PHOTON_KEYSTREAM_V1`, `DOMSEP_ORBITAL_STATE_V1`),
  ensuring cryptographic independence even when derived from the same seed.

### Violation Consequence

A break of SHAKE256 would compromise all stream cipher modes (V2, V3, H, Prism, Split,
Flare) and the entropy extraction pipeline. Note the limit of the "second line of
defense" framing: the n-body layer only protects *config recovery* (guessing the
shared secret). Against keystream recovery (known plaintext), the attacker
bypasses the simulation entirely and attacks SHAKE256 directly.

### Supporting Evidence

- **NIST FIPS PUB 202** (2015): Standard specification.
- **Bertoni et al. (2013)**: Keccak security analysis.
- **NIST PQC standardization**: SHAKE256 is a core component of ML-KEM (FIPS 203)
  and ML-DSA (FIPS 204), both of which Kelvin integrates.

---

## Assumption 4: Lyapunov Horizon Validity

### Statement

Simulations with `total_steps < min_chaos_steps` produce keystream that is
computationally indistinguishable from random. Simulation beyond the Lyapunov
horizon may degrade unpredictability as trajectory divergence saturates.

> ⚠️ **Important caveat**: A positive Lyapunov exponent (λ > 0) confirms the system operates in a chaotic regime (exponential sensitivity to initial conditions). This is a **qualitative** property. It does **not** directly measure or bound the cryptographic entropy of the output. The Kaplan-Yorke dimension (used in C2) estimates the fractal dimension of the attractor in phase space — this is a geometric quantity, not bits of min-entropy. The conversion from Lyapunov exponents to "bits of entropy" is an informal argument, not a formal reduction.

### Rationale

The Lyapunov time is the characteristic timescale for exponential divergence
of nearby trajectories. Before the Lyapunov time, the system's state is
strongly correlated with initial conditions. After sufficient Lyapunov times,
the system state is effectively random with respect to the initial configuration.

The shadow orbit method (Benettin et al., 1980) estimates the Lyapunov
exponent by running a perturbed "shadow" trajectory alongside the main
trajectory and measuring their divergence rate.

### Enforcement

- **Lyapunov estimator** (`kelvin-kdf/src/lyapunov.rs`): Uses a deterministic
  shadow orbit to estimate λ. The `estimate()` method returns `min_chaos_steps`
  — the minimum number of steps needed to reach the chaotic regime.
- **Config validation** (`kelvin/src/lib.rs`, `run_simulation_pipeline`): Configurations
  with `total_steps < min_chaos_steps` are rejected at instantiation.
- **Default steps** (`kelvin-core/src/constants.rs`: `DEFAULT_STEPS = 1_000_000`)
  is set well above typical Lyapunov times (~1000 steps for standard configs).

### Violation Consequence

If a configuration passes validation despite having insufficient steps, the
keystream may be correlated with the initial configuration beyond what is
modeled, potentially enabling cryptographic attacks. The fail-fast mechanism
at initialization prevents this by rejecting insufficient configurations.

### Supporting Evidence

- **Lyapunov analysis**: λ ≈ 0.7/yr (positive → chaotic regime; pre-`ln(10)`-fix literature quotes 0.693) for 5-body configurations. A Lyapunov time of ~1.44 yr (~1443 steps at dt ≈ 0.001) means the system amplifies trajectory divergence by a factor of e ≈ 2.718 per Lyapunov time.
- **Entropy extraction**: SHAKE256 extracts entropy from the orbital state
  after the simulation is complete, ensuring the hash is computed on a
  fully-mixed chaotic state.
- **Reference**: Benettin et al. (1980), Wolf et al. (1985) — standard
  methods for Lyapunov exponent estimation in chaotic systems.

---

## Assumption 5: HKDF-SHA512 Security

### Statement

HKDF-SHA512 (RFC 5869) is a secure extract-and-expand key derivation function.
SHA-512 provides 256-bit preimage resistance, and HKDF's extraction step
produces uniformly distributed output even from non-uniform input entropy.

### Rationale

The key schedule in Phase 3 uses HKDF-SHA512 to derive per-key material and
MAC keys from the 2048-byte entropy pool. The security of the key schedule
depends on SHA-512's PRF properties.

### Enforcement

- **Key derivation** (`kelvin-kdf/src/schedule.rs`): HKDF-SHA512 with info
  strings `kelvin-hkdf-key-v1` / `kelvin-hkdf-nonce-v1` derives encryption keys and nonces (no `DOMSEP_KEY_SCHEDULE_V1` constant exists — earlier drafts misnamed it).
- **MAC key derivation** (`kelvin/src/authenticated.rs`): HKDF-SHA512 with
  domain separator `DOMSEP_MAC_KEY_V1` derives KMAC128 authentication keys.
- **Domain separation**: Each key derivation uses a distinct domain separator,
  preventing related-key attacks across different purposes.

### Violation Consequence

A break of SHA-512 or HKDF would compromise the key schedule, allowing an
attacker to derive encryption keys from the entropy pool or bypass forward
secrecy guarantees. However, the attacker would still need to recover the
entropy pool (requiring SHAKE256 inversion) and invert the n-body simulation.

### Supporting Evidence

- **RFC 5869** (Krawczyk & Eronen, 2010): HKDF extract-and-expand specification.
- **NIST SP 800-56C Rev. 2**: NIST key derivation guidelines followed by
  Kelvin's key schedule design.

---

## Assumption 8: Session Uniqueness (Config, Nonce)

### Statement

Each cipher instance must use a unique (config, nonce) pair. The nonce is
mixed into every seed extraction, so distinct nonces yield independent
keystreams from one config — but reusing a pair reuses keystream
(two-time pad), with no further defense.

### Enforcement

- **Extraction binding** (`kelvin-kdf/src/extractor.rs`): length-prefixed
  nonce hashed after the domain separator in `feed_orbital_state`.
- **CLI defaults** (`kelvin-cli/src/main.rs`): encrypt generates a fresh
  16-byte nonce and prints it; decrypt requires `--nonce` (`--nonce=`
  selects legacy output explicitly).

### Violation Consequence

Same-pair reuse produces identical keystream prefixes across instances;
decryption of one instance's traffic with another's stream is trivially
breakable. This is a caller-discipline requirement, enforced by defaults
but overridable.

---

## Summary Table

| # | Assumption | Enforcement Location | Risk if Violated |
|:---:|---|:---|:---|
| 1 | N-body one-way | `lyapunov.rs`, `stability.rs`, `lib.rs` | Stream cipher key recovery |
| 2 | Fixed-point determinism | `fixed_math.rs`, `determinism.rs` | Cross-platform decryption failure |
| 3 | SHAKE256 security | `extractor.rs`, mode `.rs` files | Keystream prediction |
| 4 | Lyapunov horizon | `lyapunov.rs`, `lib.rs` | Correlated keystream |
| 5 | HKDF-SHA512 security | `schedule.rs`, `authenticated.rs` | Key schedule compromise |
| 6 | Secret zeroization | `Drop` impls (`schedule.rs`, `lib.rs`, flare/prism/split, `Zeroizing` buffers; verified by `zeroize_verify`) | Cold-boot / memory-disclosure recovery |
| 7 | OS CSPRNG seeding | `keygen.rs` (`rand::thread_rng`) | Generated keyspace capped at OS entropy (2²⁵⁶); weak RNG collapses it |
| 8 | Session uniqueness | Nonce bound into extraction (`extractor.rs`, `new_with_nonce`, CLI `--nonce`) | Same-(config, nonce) reuse reuses keystream (two-time pad) |

---

## References

- Benettin, G., Galgani, L., Giorgilli, A., & Strelcyn, J.-M. (1980).
  "Lyapunov Characteristic Exponents for Smooth Dynamical Systems and for
  Hamiltonian Systems." *Meccanica*, 15, 9–20.
- Bertoni, G., Daemen, J., Peeters, M., & Van Assche, G. (2013). "Keccak."
  *Advances in Cryptology — EUROCRYPT 2013*, 313–314.
- Goldberg, D. (1991). "What Every Computer Scientist Should Know About
  Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
- National Institute of Standards and Technology. (2015). "SHA-3 Standard:
  Permutation-Based Hash and Extendable-Output Functions." FIPS PUB 202.
- Poincaré, H. (1889). "Sur le problème des trois corps et les équations
  de la dynamique." *Acta Mathematica*, 13, 1–270.
- Poincaré, H. (1899). *Les Méthodes Nouvelles de la Mécanique Céleste*,
  Vol. 3. Gauthier-Villars. (Non-integrability proof cited in §A1 rationale.)
- Wolf, A., Swift, J. B., Swinney, H. L., & Vastano, J. A. (1985).
  "Determining Lyapunov Exponents from a Time Series." *Physica D*, 16(3),
  285–317.
