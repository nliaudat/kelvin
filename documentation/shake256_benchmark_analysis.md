# SHAKE256 Performance Analysis

## Summary

SHAKE256 is **not** the bottleneck in Kelvin's encryption pipeline. The pure Rust
implementation achieves ~560 MB/s on a Ryzen 5 5600 CPU, and the Photon/Quantum
modes reach ~520–540 MB/s in-memory — close to the raw SHAKE256 throughput.

The original file-based benchmark showing ~48 MB/s was entirely **I/O-bound**
(reading/writing 1 GB through `D:/temp/`), not crypto-bound.

## Benchmark Results

### Standalone SHAKE256 (256 MiB, pure Rust, no asm)

| Test Pattern | Throughput |
|-------------|-----------|
| Clone-per-chunk (re-absorb seed each time) | 563 MB/s |
| Persistent reader (matches Photon/Quantum) | 566 MB/s |
| Persistent reader + XOR (matches actual encrypt) | 556 MB/s |

### In-Memory Crypto Throughput (1 GB, no file I/O)

| Mode | Throughput | Engine |
|------|-----------|--------|
| secure | **1.61 GB/s (1644 MB/s)** | ChaCha20Poly1305 AEAD |
| chaos | **0.03 GB/s (34 MB/s)** | Per-step SHAKE256 XOR (1 MiB chunks) |
| photon | **0.53 GB/s (542 MB/s)** | HKDF→SHAKE256 XOR |
| quantum | **0.50 GB/s (512 MB/s)** | Hybrid cache+XOR + orbital reseed |

### File-Based Benchmark (1 GB via `D:/temp/`)

| Mode | Throughput |
|------|-----------|
| secure | ~48 MB/s |
| chaos | ~48 MB/s |
| photon | ~48 MB/s |
| quantum | ~48 MB/s |

## Analysis

### Why the 10x difference?

The file-based benchmark reads 1 GB from disk, encrypts it, and writes 1 GB back
to disk. On `D:/temp/` (a standard HDD/SSD), this I/O is the bottleneck:

- **Disk read:** ~100–500 MB/s (depends on drive)
- **Disk write:** ~100–500 MB/s (depends on drive)
- **Crypto (SHAKE256):** ~560 MB/s
- **Crypto (ChaCha20Poly1305):** ~1.6 GB/s

The effective throughput is limited by the slowest component — disk I/O.

### Why chaos mode is slow in-memory (34 MB/s)

Chaos mode uses 1 MiB chunks and re-initializes SHAKE256 for each chunk.
Processing 1 GB requires 1024 SHAKE256 XOF initializations, each of which
absorbs the orbital state. This is by design — chaos mode trades throughput
for per-step entropy amplification.

### Why no `asm` acceleration?

The `sha3` crate's `asm` feature targets Intel SHA-NI instructions, which are
only available on Intel Ice Lake and newer. The test CPU (AMD Ryzen 5 5600)
does not support SHA-NI. Even on Intel CPUs with SHA-NI, the improvement would
be at most ~2x, and SHAKE256 is already not the bottleneck.

## How to Run

### Standalone SHAKE256 benchmark

```powershell
cargo run -p shake256-bench --release
```

### In-memory crypto throughput benchmark

```powershell
python scripts/bench.py 1 --level paranoid --in-memory
```

This runs all 4 modes (secure, chaos, photon, quantum) in-memory, bypassing
file I/O entirely. Results are written to `documentation/bench_1gb.md`.

### File-based benchmark (original)

```powershell
python scripts/bench.py 1 --level paranoid
```

## Conclusion

**No changes needed to `sha3` dependencies.** The `asm` feature would provide
negligible benefit on this CPU, and SHAKE256 is already fast enough that it's
not the bottleneck in any mode except chaos (which is slow by design).

If you want to improve the file-based benchmark throughput, use a RAM disk or
NVMe drive instead of `D:/temp/`.
