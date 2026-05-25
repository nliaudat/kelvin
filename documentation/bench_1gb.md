# Kelvin 1 GB Encryption Benchmark

**Date:** 24.05.2026 22:13:09
**Platform:** Cross-Platform (Python)
**Test file:** 1 GB pattern b'\x00\x01'
**Integration:** Verlet (default)
**Key level:** paranoid
**Simulation steps:** 110,000 (reduced for fast benchmarking)
**Keygen output:**
```
Generated paranoid config (fast mode, 110,000 steps).
```

## In-Memory Crypto Throughput

| Mode | Throughput (GB/s) | Throughput (MB/s) |
|------|-------------------|--------------------|
| secure | 1.61 | 1644 |
| chaos | 0.03 | 34 |
| photon | 0.53 | 542 |
| quantum | 0.50 | 512 |
|---|-------------------|--------------------|
