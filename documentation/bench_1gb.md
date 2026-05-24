# Kelvin 1 GB Encryption Benchmark

**Date:** 24.05.2026 15:43:17
**Platform:** Cross-Platform (Python)
**Test file:** 1 GB pattern b'\x00\x01'
**Integration:** Verlet (default)
**Key level:** paranoid

| Mode | Operation | Time (s) | Throughput (GB/s) | Throughput (MB/s) | Verify |
|---|-----------|----------|-------------------|--------------------|--------|
| chaos | encrypt | 31.230 | 0.032 | 32.8 | PASS |
| chaos | decrypt | 31.427 | 0.032 | 32.6 | PASS |
| photon | encrypt | ERROR | 0 | 0 | FAIL |
| photon | decrypt | ERROR | 0 | 0 | FAIL |
| quantum | encrypt | ERROR | 0 | 0 | FAIL |
| quantum | decrypt | ERROR | 0 | 0 | FAIL |
|---|-----------|----------|-------------------|--------------------|--------|

*Benchmark completed at 24.05.2026 15:45:15*
