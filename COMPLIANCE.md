# Kelvin — Compliance Register

> This document replaces `bad.md` (removed 2026-10-10). Every limb of the old
> production-cipher critique is tracked below as **FIXED**, **BOUNDED**, or
> **ACCEPTED LIMITATION** with evidence. Nothing here raises the effective
> security above the stated bound: **128-bit post-quantum via SHAKE256**.

## 1. Security ceiling — BOUNDED

The keystream is SHAKE256 output; no simulation depth raises effective
security past 128-bit PQ (Grover). This is now the *stated* claim everywhere
(`README` mode table, `THREAT_MODEL.md` §3.3, `formal_verification.md` §2,
`stream_cipher_security.md` §8). The old "astronomical keyspace / 2⁹⁶⁰"
framing is removed from all normative statements; remaining 2^1920 figures
are explicitly scoped to the representable (not generated) space.

## 2. KDF cost — ACCEPTED LIMITATION

Setup costs seconds (1M steps) to minutes and is compute-bound with no
memory hardness — weaker per-dollar than Argon2id against ASIC/GPU brute
force. Mitigations in place: `keygen --fast` for non-production use,
Lyapunov-gated minimum steps (no wasted oversimulation), measured costs
published (`README` performance table). Positional statement: the cost is a
sequential work factor, not a memory-hard KDF; do not present it as one.

## 3. Nonce handling — FIXED

Session nonces are bound into every seed extraction
(`extract_*_with_nonce`, `Kelvin::new_with_nonce`,
`KelvinStreaming::new_with_nonce`, CLI `--nonce`):
- CLI encrypt generates a fresh 16-byte nonce by default and prints it.
- CLI decrypt requires `--nonce` (no silent default); `--nonce=`
  explicitly selects pre-nonce legacy output.
- Rule: one (config, nonce) pair per instance. Same-nonce reuse still
  reuses keystream (two-time pad) — documented in `usage.md`.

## 4. Known-plaintext / malleability — FIXED (default-auth)

XOR stream ciphers are malleable and known plaintext reveals keystream —
inherent, shared with all stream ciphers. Guardrail: KMAC128 authentication
is **on by default** for chaos/photon/quantum (`--no-auth` opts out);
V1 Secure is AEAD. Decrypt verifies in constant time and rejects tampering
instead of returning malleated plaintext.

## 5. Production hallmarks — status per item

| Hallmark | Status |
|----------|--------|
| Formal security reduction | ACCEPTED LIMITATION — none; C1–C5 are plausibility arguments (21 proved-step / 2 justified), not reductions |
| Independent cryptanalysis | ACCEPTED LIMITATION — none to date; adversarial chain (v1–v5) is self-review, not independent audit |
| Standardized primitives | MET where used — SHAKE256, ChaCha20Poly1305, HKDF-SHA512, BLAKE3, ML-KEM-768, ML-DSA-65 |
| Side-channel hardening | PARTIAL — DudeCT \|\|t\| < 5 on arithmetic primitives + extract/simulate/schedule (12 benches, gated in `test_all.bat`); full encrypt/decrypt pipeline not covered |
| Nonce management | FIXED — §3 above |
| Authentication | FIXED — default-on (§4 above) |
| Key setup cost | DOCUMENTED — measured table in `README`; `SECURITY.md` states experimental status |

## Residual risks (must-read before any deployment)

1. Same-(config, nonce) reuse → identical keystream (two-time pad).
2. No memory hardness — budget-constrained defenders lose to ASIC/GPU attackers.
3. No reduction, no independent audit — research-grade, not FIPS-grade.
4. Finite-precision periodicity has no runtime detector (see v1§6 / v5§7).
5. `verlet_step` timing variation (|t| ≈ 75 in one DudeCT configuration) is
   under investigation, not exonerated.
