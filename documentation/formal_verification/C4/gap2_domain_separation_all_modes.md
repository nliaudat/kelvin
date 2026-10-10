# C4 Gap 2: Domain Separation Across All 6 Modes — Resolved (corrected 2026-10-10)

> **Status:** ✅ RESOLVED (corrected)
> **Date:** 2026-05-30 (corrected 2026-10-10)
> **Conjecture:** C4 — Computational Indistinguishability of the Keystream
> **Kani Cross-Reference:** `verify_domain_separation_functional` in `kelvin-kdf/src/extractor.rs`

## 1. Theorem Statement

For each pair of distinct domain separators `(d_i, d_j)` used by the 6
encryption modes, the SHAKE256 outputs for the same orbital state differ
with overwhelming probability (assuming SHAKE256 collision resistance —
distinct outputs do not hold unconditionally, by pigeonhole):

$$SHAKE256(\text{state}, d_i) \neq SHAKE256(\text{state}, d_j) \quad \text{w.o.p.}$$

## 2. Domain Separators

Actual constants (`kelvin/src/parameters.rs`, plus two hardcoded):

| Mode | Keystream separator | Reseed / aux separators | Status |
|------|--------------------|------------------------|--------|
| Chaos V2 | `kelvin-streaming-v2-v1-000000000` (hardcoded `lib.rs`) | `kelvin-streaming-mac-key-v1` (auth) | ✅ Distinct |
| Photon V3 | `kelvin-photon-keystream-v1` | `kelvin-photon-reseed-v1` | ✅ Distinct |
| Quantum H | `kelvin-quantum-keystream` | `kelvin-quantum-seed-perturb-v1`, `kelvin-quantum-reseed-v1`, `kelvin-quantum-cache-v1` | ✅ Distinct |
| Prism | `kelvin-prism-keystream-v1` | `kelvin-prism-reseed-v1` | ✅ Distinct |
| Split | `kelvin-split-keystream-v1` | `kelvin-split-reseed-v1` | ✅ Distinct |
| Flare | `kelvin-flare-keystream-v1` | `kelvin-flare-reseed-v1`, `kelvin-flare-xof-v1` (hardcoded `flare.rs`) | ✅ Distinct |

> **Correction (2026-10-10):** the previous revision listed six
> placeholder strings (`kelvin-chaos-v2`, …) occurring nowhere in the
> codebase. All 15 pairwise combinations (C(6,2), not "6 pairs") use
> distinct byte strings above. The Kani harness
> `verify_domain_separation_functional` proves the property for arbitrary
> distinct separators, which generalizes to all 15 pairs.

## 3. References

- `kelvin-kdf/src/extractor.rs` lines 132–166 (Kani harness)
- `kelvin/src/mode.rs` (domain separator definitions for each mode)
---

## See Also

- [C4 Proof Sketch](readme.md)
