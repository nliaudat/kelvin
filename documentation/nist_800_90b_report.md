# NIST SP 800-90B Entropy Source Validation Report

**Cryptosystem:** Kelvin — Orbital Chaos KDF  
**Version:** 0.1.0  
**Date:** {{DATE}}  
**Prepared by:** {{AUTHOR}}

---

## 1. Entropy Source Overview

### 1.1 Noise Source

| Property | Value |
|----------|-------|
| **Noise source type** | Deterministic chaotic simulation (n-body gravitational) |
| **Algorithm** | Symplectic Verlet / Explicit Euler integration |
| **State size** | 5 bodies × 7 fields (mass, position×3, velocity×3) = 35 × Q32.64 fixed-point values |
| **Initial entropy** | Random orbital configuration (Sun mass ±25%, planet positions/velocities) |
| **Chaos mechanism** | Lyapunov exponential divergence of trajectories |

### 1.2 Conditioning Component

| Property | Value |
|----------|-------|
| **Conditioning function** | SHAKE256 (Extendable-Output Function) |
| **NIST standard** | FIPS 202 / SP 800-185 |
| **Input** | Orbital state (positions, velocities, masses, accelerations, G, softening, step counter, domain separator) |
| **Output length** | Configurable (default: 65,536 bytes per step) |
| **Domain separation** | `b"kelvin-streaming-v2-v1-000000000"` |
| **Security strength** | 256 bits (classical) / 128 bits (quantum) |

### 1.3 Entropy Source Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Noise Source (Orbital Simulation)             │
│                                                                  │
│  Initial Conditions ──→ n-body integrator ──→ Chaotic State      │
│  (random config)         (Verlet/Euler)       (positions,        │
│                                                 velocities,      │
│                                                 accelerations)   │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Conditioning Component (SHAKE256)                │
│                                                                  │
│  feed_orbital_state(hasher, bodies, step, G, softening,         │
│                     domain_separator)                            │
│    ├── Domain separation                                         │
│    ├── Physical constants (G, softening)                         │
│    ├── Step counter                                              │
│    ├── Number of bodies                                          │
│    ├── Instantaneous accelerations                               │
│    └── For each body: mass, position (x,y,z), velocity (x,y,z), │
│                        acceleration (x,y,z)                      │
│                                                                  │
│  output = SHAKE256(hasher.finalize(), output_len)                │
└──────────────────────────────┬──────────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Raw Keystream Output                          │
│                                                                  │
│  XOR(plaintext, keystream) → ciphertext                          │
│  (no additional cipher layer — raw SHAKE256 output)              │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. Test Methodology

### 2.1 Test Data

| Property | Value |
|----------|-------|
| **Sample size** | 1,073,741,824 bytes (1 GiB) |
| **Sample format** | Raw binary (no headers, no framing) |
| **Generation tool** | `cargo run -p nist_800_90b -- generate --size 1073741824 --output keystream.bin` |
| **Orbital config** | Default 5-body (see §2.2) |
| **Integration** | Euler (maximum chaos amplification) |
| **Bytes per step** | 65,536 (64 KiB) |
| **Total simulation steps** | 16,384 |

### 2.2 Orbital Configuration

```json
{
  "bodies": [
    {"mass": 1.0, "position": [0, 0, 0], "velocity": [0, 0, 0]},
    {"mass": 5.42e-7, "position": [1, 0, 0], "velocity": [0, 6, 0]},
    {"mass": 2.71e-7, "position": [0, 2, 0], "velocity": [-4, 0, 0]},
    {"mass": 1.36e-7, "position": [-1, -1, 0], "velocity": [3, -2, 0]},
    {"mass": 6.78e-8, "position": [2, -1, 1], "velocity": [-2, 3, 0]}
  ],
  "total_steps": 1000,
  "dt": 5.42e-7,
  "softening": 2.71e-6,
  "g": 39.478
}
```

### 2.3 Tests Performed

| Test | Standard | Description |
|------|----------|-------------|
| Shannon Entropy | — | Per-byte entropy estimate (target: >7.5 bits/byte) |
| Adjacent-byte Correlation | — | Pearson correlation coefficient (target: <0.01) |
| Chi-square Distribution | — | Byte value uniformity (target: χ² < 310, df=255) |
| Repetition Test | SP 800-90B §4.4.1 | Maximum consecutive identical bytes (target: ≤5) |
| Adaptive Proportion Test | SP 800-90B §4.4.2 | Worst-case byte frequency in sliding window (target: ≤16/512) |
| Runs Test | SP 800-22 §2.3 | Bit-level run count vs. expected (target: |z| < 2.576) |
| Longest Run Test | SP 800-22 §2.4 | Longest consecutive identical bits (target: ≤26) |
| NIST ea_iid | SP 800-90B §5 | Official NIST min-entropy estimation tool |

---

## 3. Results

### 3.1 Built-in Health Tests

| Test | Result | Value | Threshold | Status |
|------|--------|-------|-----------|--------|
| Shannon Entropy | {{RESULT}} | {{VALUE}} bits/byte | >7.5 | {{PASS/FAIL}} |
| Adjacent-byte Correlation | {{RESULT}} | {{VALUE}} | <0.01 | {{PASS/FAIL}} |
| Chi-square Distribution | {{RESULT}} | {{VALUE}} | <310 | {{PASS/FAIL}} |
| Repetition Test (§4.4.1) | {{RESULT}} | max {{VALUE}} consecutive | ≤5 | {{PASS/FAIL}} |
| Adaptive Proportion Test (§4.4.2) | {{RESULT}} | worst {{VALUE}}/512 | ≤16 | {{PASS/FAIL}} |
| Runs Test (§2.3) | {{RESULT}} | {{VALUE}} runs | |z| < 2.576 | {{PASS/FAIL}} |
| Longest Run Test (§2.4) | {{RESULT}} | longest {{VALUE}} bits | ≤26 | {{PASS/FAIL}} |

### 3.2 NIST ea_iid Results

| Metric | Value |
|--------|-------|
| **H_IID** (IID min-entropy estimate) | {{VALUE}} bits/byte |
| **H_non-IID** (non-IID min-entropy estimate) | {{VALUE}} bits/byte |
| **H_min** (final min-entropy estimate) | {{VALUE}} bits/byte |
| **H_min per 64 KB output** | {{VALUE}} bits |
| **Passed restart tests** | {{YES/NO}} |

### 3.3 Conditioning Assessment (NIST SP 800-90C)

| Requirement | Status | Notes |
|-------------|--------|-------|
| Conditioning function is NIST-approved | ✅ | SHAKE256 (FIPS 202) |
| Security strength ≥ 256 bits | ✅ | SHAKE256 provides 256-bit classical security |
| Output length ≤ input entropy × security factor | ✅ | 64 KB output per step |
| Domain separation applied | ✅ | `b"kelvin-streaming-v2-v1-000000000"` |

---

## 4. Conclusion

{{CONCLUSION}}

### 4.1 FIPS 140-3 Submission Readiness

| Criterion | Status |
|-----------|--------|
| Entropy source validated per SP 800-90B | {{YES/NO}} |
| Conditioning component documented per SP 800-90C | {{YES/NO}} |
| Health tests implemented and passing | {{YES/NO}} |
| Continuous health test monitoring | {{YES/NO}} |
| Cryptographic module boundary defined | {{YES/NO}} |

### 4.2 Recommendations

{{RECOMMENDATIONS}}

---

## Appendix A: Raw Test Output

```
{{PASTE ea_iid.py output here}}
```

## Appendix B: Keystream Generation Command

```
cargo run --release -p nist_800_90b -- generate --size 1073741824 --output keystream_1gb.bin
```

## Appendix C: Analysis Command

```
cargo run --release -p nist_800_90b -- analyze --input keystream_1gb.bin
```
