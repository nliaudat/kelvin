# Kelvin — formal models

Placeholder models used to smoke-test the toolchain in `docker/formal/`. They
are deliberately trivial.

| File | Tool | What it checks |
|------|------|----------------|
| `proverif/kelvin_smoke.pv` | ProVerif | a private key used only through a hash is not derivable |
| `tamarin/kelvin_smoke.spthy` | Tamarin | a generated key is never learned by the adversary |
| `tamarin/nonce_reuse.spthy` | Tamarin | reusing `(config, nonce)` leaks the message (expected **falsified**) |
| `tamarin/nonce_fresh.spthy` | Tamarin | fresh nonces keep the message secret (expected **verified**) |
| `easycrypt/smoke.ec` | EasyCrypt | the prover and SMT backend run |

The two `nonce_*` models are a matched pair: the first demonstrates the
two-time-pad attack documented in `COMPLIANCE.md`, the second shows that nonce
freshness removes it (given the PRF assumption on `h`).

## Writing a real Kelvin model

These files are scaffolds. A meaningful model would need, at minimum:

1. an explicit handshake that establishes the orbital configuration (the secret
   input), because Kelvin is symmetric-only;
2. nonce generation and uniqueness, since `(config, nonce)` reuse is a two-time
   pad (see `COMPLIANCE.md` §3);
3. the domain-separation tags used by `kelvin-kdf/src/extractor.rs`;
4. an explicit leakage/compromise model for the orbital state.

Until that model exists, no protocol-level claim about Kelvin is proved.
