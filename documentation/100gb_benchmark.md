# Kelvin 100 GB Encryption Benchmark 
 
**Date:** 24.05.2026 11:02:55,38 
**Platform:** Windows 
**Test file:** 100 GB alternating 0x00/0x01 pattern 
**Integration:** Euler (default) 
**Key level:** standard (5 bodies, 1M steps) 
 
| Mode | Operation | Time (s) | Throughput (GB/s) | Verify | 
|---|-----------|----------|-------------------|--------| 
| chaos | encrypt | 111425 |  | PASS | 
| chaos | decrypt | 111425 |  | PASS | 
| photon | encrypt | 111425 |  | FAIL | 
| photon | decrypt | 111425 |  | FAIL | 
| quantum | encrypt | 111425 |  | FAIL | 
| quantum | decrypt | 111425 |  | FAIL | 
|---|-----------|----------|-------------------|--------| 
 
*Benchmark completed at 24.05.2026 11:02:58,48* 
