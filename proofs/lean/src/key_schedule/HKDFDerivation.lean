/-
# HKDF key derivation (legacy path)

Corrected 2026-11-23. The previous revision claimed forward secrecy and key
uniqueness as proved. Forward secrecy is a protocol property and is not modelled
here.

## What is proved

* key derivation is deterministic;
* distinct indices produce distinct info strings;
* **if** the PRF is injective in its index argument, distinct indices produce
  distinct keys.

## What is not proved

Injectivity of HMAC-SHA512 (a cryptographic assumption), forward secrecy, and
reseed-interval safety.
-/
import KelvinProofs.KeySchedule

namespace KelvinLegacy.KeySchedule

open KelvinProofs.KeySchedule

/-- Key derivation is deterministic. -/
theorem derive_deterministic (prf : PRF) (pool : List Nat) (i : Nat) :
    deriveKey prf pool i = deriveKey prf pool i := rfl

/-- Distinct indices produce distinct info strings. -/
theorem infoString_injective (tag : List Nat) (i j : Nat)
    (h : infoString tag i = infoString tag j) : i = j :=
  KelvinProofs.KeySchedule.infoString_injective tag i j h

/-- **Conditional** key uniqueness: assuming the PRF is injective in its index
argument, distinct indices produce distinct keys. -/
theorem key_unique_of_injective (prf : PRF)
    (hinj : ∀ pool a b, prf pool a = prf pool b → a = b)
    (pool : List Nat) (i j : Nat) (hij : i ≠ j) :
    deriveKey prf pool i ≠ deriveKey prf pool j :=
  KelvinProofs.KeySchedule.key_unique_of_injective prf hinj pool i j hij

end KelvinLegacy.KeySchedule
