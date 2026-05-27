# Q32.64 Fixed-Point Arithmetic Specification

## Format

Kelvin uses Q32.64 fixed-point arithmetic stored as `i128`:

- **Integer bits**: 32 (supports values up to ~±4.29 × 10⁹)
- **Fractional bits**: 64 (resolution ~5.4 × 10⁻²⁰)
- **Scaling factor**: 2⁶⁴
- **Representation**: `value = raw / 2⁶⁴`

## Constants

| Constant | Mathematical Value | Raw Q32.64 Value |
|----------|-------------------|------------------|
| `Fixed::ZERO` | 0 | `0` |
| `Fixed::ONE` | 1 | `1 << 64` |
| `Fixed::MAX` | ~1.70 × 10³⁸ | `i128::MAX` |
| `Fixed::MIN` | ~-1.70 × 10³⁸ | `i128::MIN` |

## Operations

### Addition

**Specification**: `Fixed::add(a, b) = a + b`

For inputs `a, b ∈ [-100, 100] AU`:
- Maximum result: 200 AU → raw = `200 << 64 ≈ 3.69 × 10²¹`
- i128 capacity: ~1.70 × 10³⁸ → **17 orders of magnitude headroom**
- No overflow possible within physical bounds

**Implementation**: Uses Rust's `i128::wrapping_add` with overflow detection.
Panics on overflow in debug mode; wraps in release mode (but overflow is
provably impossible within physical bounds).

### Subtraction

**Specification**: `Fixed::sub(a, b) = a - b`

For inputs `a, b ∈ [-100, 100] AU`:
- Minimum result: -200 AU → raw = `-200 << 64`
- Same headroom as addition

### Multiplication

**Specification**: `Fixed::mul(a, b) = a × b`

The multiplication uses a constant-time 3-split algorithm:

1. Split each operand into high/low u64 halves
2. Compute four partial products: `hi_hi`, `hi_lo`, `lo_hi`, `lo_lo`
3. Combine with appropriate shifts

**Error bound**: ≤ 1 ULP (unit in the last place)

For inputs `a, b ∈ [-100, 100] AU`:
- Maximum product: 10,000 AU² → raw = `10000 << 64 ≈ 1.84 × 10²³`
- Well within i128 range

### Division

**Specification**: `Fixed::div(a, b) = a / b`

Uses a constant-time 192-iteration restoring division algorithm:

1. Shift numerator left by 64 bits
2. For each of 192 iterations:
   - Subtract denominator from working value
   - If result is non-negative, set quotient bit
   - Otherwise, restore previous value
3. Result is the quotient

**Error bound**: ≤ 1 ULP

**Preconditions**:
- `b ≠ 0` (panics on division by zero)
- `|a| ≤ 40 << 64` (G * mass bound)
- `|b| ≥ 1 << 24` (softening bound)

### Square Root

**Specification**: `Fixed::sqrt(a) = √a` for `a ≥ 0`

Uses a constant-time binary digit-by-digit (restoring) algorithm:

1. Process 128-bit input 2 bits at a time
2. Build 96-bit result (32 integer + 64 fractional bits)
3. Exactly 96 iterations regardless of input magnitude
4. For `a < 0`, returns 0 via constant-time masking

**Error bound**: ≤ 1 ULP

**Preconditions**:
- Input `a ∈ [0, (200 AU)²]` for simulation use

## Physical Bounds

The simulation operates within these physical bounds:

| Parameter | Min | Max | Units |
|-----------|-----|-----|-------|
| Position | -100 | 100 | AU |
| Mass | 10⁻⁶ | 1 | M☉ |
| G | 1.0 | 1000.0 | AU³/(M☉·yr²) |
| Timestep (dt) | 10⁻⁸ | 10⁻¹ | yr |
| Softening (ε) | — | ~9.5 × 10⁻⁷ | AU |
| Separation | ~9.3 × 10⁻⁸ | 200 | AU |

## Invariants

1. **Determinism**: All operations produce bit-identical results on all
   supported architectures (x86_64, aarch64, wasm32).
2. **Constant-time**: No operation has data-dependent timing (verified by
   dudect-bencher with |t| < 5 for all individual operations).
3. **No overflow**: Within physical bounds, no operation can overflow i128.
4. **No panic**: Within physical bounds, no operation can panic (division by
   zero is prevented by softening and collapse detection).

## References

- Goldberg, D. (1991). "What Every Computer Scientist Should Know About
  Floating-Point Arithmetic." *ACM Computing Surveys*, 23(1), 5–48.
- Apple Security Research (2026). "Formal verification of corecrypto for
  post-quantum cryptography." — Blueprint for proving functional equivalence
  of cryptographic implementations against mathematical specifications.
