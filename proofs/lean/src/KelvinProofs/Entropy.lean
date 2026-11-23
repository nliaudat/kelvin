/-
# Entropy extraction (abstract model)

Models domain-separated framing of the entropy pool before it is passed to
SHAKE256.

## What is proved

* distinct domain tags produce distinct framed inputs;
* **if** the XOF is injective, distinct tags produce distinct outputs.

## What is not proved

SHAKE256 preimage/collision resistance is a cryptographic assumption. It is not
formalised here, and no claim about "bits of entropy lost per step" follows from
this file.
-/
import Init

namespace KelvinProofs.Entropy

/-- A XOF (e.g. SHAKE256) modelled as an opaque function on framed inputs. -/
abbrev XOF := List Nat → List Nat

/-- Domain-separated framing: the tag is prepended to the pool. -/
def frame (tag : Nat) (pool : List Nat) : List Nat := tag :: pool

/-- Distinct domain tags produce distinct framed inputs. -/
theorem domain_separation (pool : List Nat) (t₁ t₂ : Nat) (h : t₁ ≠ t₂) :
    frame t₁ pool ≠ frame t₂ pool := by
  intro hc
  exact h (by simpa [frame] using hc)

/-- **Conditional** output separation: assuming the XOF is injective, distinct
domain tags produce distinct outputs. -/
theorem xof_separates_domains (xof : XOF)
    (hinj : ∀ a b, xof a = xof b → a = b)
    (pool : List Nat) (t₁ t₂ : Nat) (h : t₁ ≠ t₂) :
    xof (frame t₁ pool) ≠ xof (frame t₂ pool) := by
  intro hc
  exact domain_separation pool t₁ t₂ h (hinj _ _ hc)

end KelvinProofs.Entropy
