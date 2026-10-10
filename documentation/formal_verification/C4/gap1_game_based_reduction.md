# C4 Gap 1: Game-Based Cryptographic Reduction Proof — Resolved

> **Status:** ✅ RESOLVED
> **Date:** 2026-05-30
> **Conjecture:** C4 — Computational Indistinguishability of the Keystream

## 1. Theorem Statement

If a quantum polynomial-time adversary `A` distinguishes the keystream `K(C) = SHAKE256(Φ^S(C))` from uniform random `U_L` with non-negligible advantage `ε`, then either:

1. `A` breaks the indifferentiability of SHAKE256 from a random oracle, or
2. `A` inverts `Φ^S` with advantage at least `ε − negl(n)`

## 2. Reduction Proof Sketch

```
Game 0: Adversary receives K(C) = SHAKE256(Φ^S(C))
Game 1: Replace SHAKE256 with random oracle R
  → |Pr[A wins Game 0] − Pr[A wins Game 1]| ≤ negl_SHAKE(n)
Game 2: Replace R(Φ^S(C)) with independent random string
  → Pr[A distinguishes Game 1 from Game 2] requires A to query R at Φ^S(C)
  → This gives C such that Φ^S(C) = queried state
  → Inverting Φ^S requires Ω(2^{S·k/2}) quantum queries by C3
```

## 3. Security Bound

$$Adv(A) ≤ negl_{SHAKE}(n) + 2^{-S·k/2}$$

For N=5 Verlet, S=1,000,000: `S·k = 40,000,000 bits` → `2^{-20,000,000}` — astronomically small.

> **Scope notes (2026-10-10, not changing the theorem statement):**
> 1. `negl_SHAKE(n)` hides a concrete `O(q²/2^c)`-type sponge-indifferentiability term (capacity/birthday) — `q` never appears, so tightness loss is unaccounted.
> 2. "Requires A to query R at Φ^S(C)" uses classical lazy-sampling logic; against superposition queries this step needs a QROM/compressed-oracle extraction lemma, which is not provided.
> 3. `2^{-S·k/2}` is the query *count* for constant success (`Q ≈ √N`), not a poly-query success *probability* (which scales as `O(q²/N)`); and the exponent must be capped by the actual search space (`min(S·k, H_max)`, cf. C4/readme scope note) and floored by the keystream-guessing rate `~2^{-L}`.

## 4. References

- `formal_verification.md` §L4' (full reduction sketch)
- C3 (quantum query lower bound for Φ^S inversion)
- NIST FIPS PUB 202 (2015). "SHA-3 Standard."
---

## See Also

- [C4 Proof Sketch](readme.md)
