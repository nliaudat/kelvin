# Kelvin 100 GB Encryption Benchmark 
 
**Date:** 24.05.2026  9:56:47,35 
**Platform:** Windows 
**Test file:** 100 GB alternating 0x00/0x01 pattern 
**Integration:** Euler (default) 
**Key level:** standard (5 bodies, 1M steps) 
 
| Mode | Operation | Time (s) | Throughput (GB/s) | Verify | 
|---|-----------|----------|-------------------|--------| 
| chaos | encrypt | 111793 |  | FAIL | 
| chaos | decrypt | 111793 |  | FAIL | 
| photon | encrypt | ERROR | 0 | FAIL | 
| photon | decrypt | 111793 |  | FAIL | 
| quantum | encrypt | 111793 | 0 | FAIL | 
| quantum | decrypt | 111793 |  | FAIL | 
|---|-----------|----------|-------------------|--------| 
 
*Benchmark completed at 24.05.2026 10:05:31,12* 
