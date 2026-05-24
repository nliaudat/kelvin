# Kelvin 1 GB Encryption Benchmark

**Date:** 24.05.2026 16:33:34
**Platform:** Cross-Platform (Python)
**Test file:** 1 GB pattern b'\x00\x01'
**Integration:** Verlet (default)
**Key level:** paranoid
**Keygen output:**
```
Generated paranoid config.
```

| Mode | Operation | Time (s) | Throughput (GB/s) | Throughput (MB/s) | Verify |
|---|-----------|----------|-------------------|--------------------|--------|
| chaos | encrypt | 30.157 | 0.033 | 34.0 | PASS |
| chaos | decrypt | 30.199 | 0.033 | 33.9 | PASS |
| photon | encrypt | 354.634 | 0.003 | 2.9 | PASS |
| photon | decrypt | 403.049 | 0.002 | 2.5 | PASS |
