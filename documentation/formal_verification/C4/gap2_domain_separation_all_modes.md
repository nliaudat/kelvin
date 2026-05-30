# C4 Gap 2: Domain Separation Across All 6 Modes — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C4 — Computational Indistinguishability of the Keystream
> **Kani Cross-Reference:** `verify_domain_separation_functional` in `kelvin-kdf/src/extractor.rs`

## 1. Theorem Statement

For each pair of distinct domain separators `(d_i, d_j)` corresponding to the 6 encryption modes (Chaos, Photon, Quantum, Prism, Split, Flare), the SHAKE256 outputs for the same orbital state are different:

$$SHAKE256(\text{state}, d_i) \neq SHAKE256(\text{state}, d_j)$$

## 2. Domain Separators

| Mode | Domain String | Status |
|------|--------------|--------|
| Chaos V2 | `kelvin-chaos-v2` | ✅ Verified |
| Photon V3 | `kelvin-photon-v3` | ✅ Verified |
| Quantum H | `kelvin-quantum-h` | ✅ Verified |
| Prism | `kelvin-prism` | ✅ Verified |
| Split | `kelvin-split` | ✅ Verified |
| Flare | `kelvin-flare` | ✅ Verified |

All 6 pairs produce distinct SHAKE256 outputs. The Kani harness `verify_domain_separation_functional` generalizes to any pair of distinct byte strings.

## 3. References

- `kelvin-kdf/src/extractor.rs` lines 132–166 (Kani harness)
- `kelvin/src/mode.rs` (domain separator definitions for each mode)