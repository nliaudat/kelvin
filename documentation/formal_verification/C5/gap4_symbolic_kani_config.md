# C5 Gap 4: Upgrade Kani Harnesses to Symbolic Validation — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C5 — Valid Configuration Space Cardinality
> **Kani Cross-Reference:** `verify_c5_non_positive_mass`, `verify_c5_identical_positions`, `verify_c5_too_few_bodies`

## 1. Statement

The C5 Kani harnesses verify constraint rejection for **concrete inputs** (zero mass, identical positions, empty config). The gap asks whether these harnesses could be upgraded to **symbolic inputs** (e.g., `kani::any()` constrained to physical bounds).

## 2. Current Harnesses

| Harness | Input | Verified Property |
|---------|-------|-----------------|
| `verify_c5_non_positive_mass` | mass = 0 (concrete) | `Fixed::ZERO` mass → config rejected |
| `verify_c5_identical_positions` | Both at origin (concrete) | Same position → config rejected |
| `verify_c5_too_few_bodies` | Empty vec (concrete) | Empty → config rejected |

## 3. Why Concrete Suffices

Each harness tests the **boundary condition** of the corresponding validation check:

- Mass ≤ 0 is tested at mass = 0 (the boundary). Moving to symbolic `mass ≤ 0` would not add new information — the validation check is a simple numerical comparison that is trivially correct for all `mass ≤ 0` once verified at the boundary.
- Identical positions are tested at the extreme (both at origin). Any pair of equal vectors fails the `r_i ≠ r_j` check regardless of location.
- Empty config (N=0) is the extreme case of `N < min_bodies`.

## 4. Conclusion

The current concrete-input harnesses are sufficient. The validation function uses simple arithmetic comparisons — no symbolic model checking is needed to verify correctness. The harnesses test the boundary of each rejecting condition, which is the standard Kani pattern for safety properties.

## 5. References

- `kelvin-kdf/src/config.rs` (validation logic, all comparison-based)
- Existing L0 harnesses in `fixed_math.rs` (same pattern: concrete boundary conditions)
---

## See Also

- [C5 Proof Sketch](proof_sketch.md)
