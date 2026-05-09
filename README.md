# Kelvin — Orbital Chaos KDF Cryptosystem

**EXPERIMENTAL — NOT FOR PRODUCTION USE.**

Kelvin is an experimental cryptosystem that derives cryptographic keys from the chaotic evolution of an n-body gravitational system. It combines:

- **Q32.64 fixed-point arithmetic** — deterministic across all platforms
- **Symplectic Verlet integrator** — energy-conserving n-body simulation
- **Lyapunov time estimation** — shadow orbit method for chaos quantification
- **SHA3-512 entropy extraction** — domain-separated hashing of orbital state
- **ChaCha20 stream cipher** — XOR-based encryption/decryption

## Architecture

```
kelvin-core/     — Fixed-point math, Vec3, OrbitalBody, Verlet integrator
kelvin-kdf/      — OrbitalConfig, LyapunovEstimator, SHA3-512 extractor, KeySchedule
kelvin-stream/   — ChaCha20 wrapper with StreamCipher trait
kelvin/          — Top-level Kelvin struct (encrypt/decrypt)
kelvin-cli/      — CLI tool (keygen, encrypt, decrypt, benchmark)
kelvin-ffi/      — C FFI bindings for iOS/Android/embedded
```

## Quick Start

```rust
use kelvin::{Kelvin, OrbitalConfig, Fixed, Vec3, OrbitalBody};

// 1. Create orbital configuration (shared secret)
let config = OrbitalConfig::new(
    vec![
        OrbitalBody::new(Fixed::ONE, Vec3::ZERO, Vec3::ZERO),
        OrbitalBody::new(
            Fixed::from_raw(1 << 54),
            Vec3::new(Fixed::ONE, Fixed::ZERO, Fixed::ZERO),
            Vec3::new(Fixed::ZERO, Fixed::from_int(6), Fixed::ZERO),
        ),
    ],
    1_000_000,  // total simulation steps
    10_000,     // reseed interval
    Fixed::from_raw(1 << 44),  // time step (~1e-6 years)
    Fixed::from_raw(1 << 44),  // softening factor
)?;

// 2. Create Kelvin instance (runs simulation + Lyapunov estimation)
let mut k = Kelvin::new(config)?;

// 3. Encrypt/decrypt (XOR with keystream)
let mut data = b"Hello, world!".to_vec();
k.encrypt(&mut data)?;
k.decrypt(&mut data)?;
assert_eq!(&data, b"Hello, world!");
```

## Security Levels

| Level    | Bodies | Steps     | Lyapunov Shadow | Setup Time |
|----------|--------|-----------|-----------------|------------|
| Standard | 3      | 1,000,000 | 1,000           | ~1s        |
| Paranoid | 5      | 10,000,000| 10,000          | ~10s       |
| Maximum  | 10     | 100,000,000| 100,000        | ~2min      |

## References

See [REFERENCES.bib](REFERENCES.bib) for the full BibTeX bibliography. Key references include:

- **Verlet (1967)** — Original leapfrog integration method
- **Benettin et al. (1980)** — Lyapunov exponent computation
- **Wolf et al. (1985)** — Shadow orbit method for Lyapunov estimation
- **Bernstein (2008)** — ChaCha20 stream cipher specification
- **NIST FIPS PUB 202 (2015)** — SHA3-512 standard
- **CryptoChaos (Harvard, 2025)** — Related chaos-based cryptographic framework

## License

Licensed under the Creative Commons Attribution-NonCommercial-ShareAlike 4.0 International License. See [licence.md](licence.md) for details.

## Citation

If you use Kelvin in academic work, please cite:

```bibtex
@misc{liaudat2025kelvin,
  author    = {Nicolas Liaudat},
  title     = {{Kelvin}: Orbital Chaos {KDF} Cryptosystem},
  year      = {2025},
  howpublished = {\url{https://github.com/nliaudat/kelvin}}
}
```
