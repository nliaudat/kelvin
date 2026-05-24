# Kelvin 1 GB Encryption Benchmark

**Date:** 24.05.2026 21:00:08
**Platform:** Cross-Platform (Python)
**Test file:** 1 GB pattern b'\x00\x01'
**Integration:** Verlet (default)
**Key level:** paranoid
**Simulation steps:** 110,000 (reduced for fast benchmarking)
**Keygen output:**
```
Generated paranoid config.
```

| Mode | Operation | Time (s) | Throughput (GB/s) | Throughput (MB/s) | Verify |
|---|-----------|----------|-------------------|--------------------|--------|
| secure | encrypt | 19.806 | 0.050 | 51.7 | PASS |
| secure | decrypt | 19.799 | 0.051 | 51.7 | PASS |
| chaos | encrypt | 33.116 | 0.030 | 30.9 | PASS |
| chaos | decrypt | 33.551 | 0.030 | 30.5 | PASS |
| photon | encrypt | 22.787 | 0.044 | 44.9 | PASS |
| photon | decrypt | 22.789 | 0.044 | 44.9 | PASS |
| quantum | encrypt | 22.816 | 0.044 | 44.9 | PASS |
| quantum | decrypt | 22.750 | 0.044 | 45.0 | PASS |
|---|-----------|----------|-------------------|--------------------|--------|

*Benchmark completed at 24.05.2026 21:03:30*

## Summary

| Mode | Encrypt | Decrypt | Verify |
|------|---------|---------|--------|
| secure | PASS | PASS | PASS |
| chaos | PASS | PASS | PASS |
| photon | PASS | PASS | PASS |
| quantum | PASS | PASS | PASS |
