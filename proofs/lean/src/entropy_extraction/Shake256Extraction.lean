/-
# SHAKE256 extraction (legacy path)

Corrected 2026-11-23. The previous revision claimed "preimage resistance",
"one-way property" and "uniform distribution" as proved, over an unclosed doc
block. None of those is provable in this model.

## What is proved

* extraction is a deterministic function of its input;
* distinct domain tags produce distinct framed inputs;
* **if** the XOF is injective, distinct tags produce distinct outputs.

## What is not proved

SHAKE256 preimage resistance, collision resistance, output uniformity, and any
statement about bits of entropy per step. Those are assumptions.
-/
import KelvinProofs.Entropy

namespace KelvinLegacy.Extraction

open KelvinProofs.Entropy

/-- Extraction is deterministic: the same framed input gives the same output. -/
theorem extraction_deterministic (xof : XOF) (pool : List Nat) :
    xof pool = xof pool := rfl

/-- Distinct domain tags produce distinct framed inputs. -/
theorem domain_separation (pool : List Nat) (t₁ t₂ : Nat) (h : t₁ ≠ t₂) :
    frame t₁ pool ≠ frame t₂ pool :=
  KelvinProofs.Entropy.domain_separation pool t₁ t₂ h

/-- **Conditional** output separation: assuming the XOF is injective, distinct
domain tags produce distinct outputs. -/
theorem xof_separates_domains (xof : XOF)
    (hinj : ∀ a b, xof a = xof b → a = b)
    (pool : List Nat) (t₁ t₂ : Nat) (h : t₁ ≠ t₂) :
    xof (frame t₁ pool) ≠ xof (frame t₂ pool) :=
  KelvinProofs.Entropy.xof_separates_domains xof hinj pool t₁ t₂ h

end KelvinLegacy.Extraction
