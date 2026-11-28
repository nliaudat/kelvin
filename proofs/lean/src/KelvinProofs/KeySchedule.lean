/-
# Key schedule (abstract model)

Models the *structure* of `kelvin-kdf`'s HKDF-based schedule: a PRF keyed by the
entropy pool and parameterised by a per-index info string.

## What is proved

* distinct indices produce distinct info strings (pure list structure);
* **if** the PRF is injective in its index argument, distinct indices produce
  distinct keys.

## What is not proved

Injectivity of HMAC-SHA512 / the PRF is a cryptographic assumption. This file
does not prove it and must not be cited as evidence for it.
-/
import Init

namespace KelvinProofs.KeySchedule

/-- A PRF modelled as an opaque function from `(pool, index)` to key bytes. -/
abbrev PRF := List Nat → Nat → List Nat

/-- Key derivation: the per-index info string is fed to the PRF. -/
def deriveKey (prf : PRF) (pool : List Nat) (index : Nat) : List Nat :=
  prf pool index

/-- The info string encodes the index. -/
def infoString (tag : List Nat) (index : Nat) : List Nat := tag ++ [index]

/-- Distinct indices give distinct info strings. -/
theorem infoString_injective (tag : List Nat) (i j : Nat)
    (h : infoString tag i = infoString tag j) : i = j := by
  have h' : tag ++ [i] = tag ++ [j] := h
  have h'' := List.append_cancel_left h'
  simpa using h''

/-- **Conditional** key uniqueness: assuming the PRF is injective in its index
argument on a fixed pool, distinct indices yield distinct keys. -/
theorem key_unique_of_injective (prf : PRF)
    (hinj : ∀ pool a b, prf pool a = prf pool b → a = b)
    (pool : List Nat) (i j : Nat) (hij : i ≠ j) :
    deriveKey prf pool i ≠ deriveKey prf pool j := by
  intro h
  exact hij (hinj pool i j h)

end KelvinProofs.KeySchedule
