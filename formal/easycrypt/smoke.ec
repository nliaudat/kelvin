(* Kelvin - EasyCrypt smoke test.

   TOOLCHAIN smoke test, not a proof about Kelvin. If this script checks, the
   EasyCrypt + Why3 + SMT stack is wired correctly.
*)

require import AllCore.

lemma smoke : 1 = 1.
proof.
  trivial.
qed.
