# Kelvin 1 GB Encryption Benchmark 
 
**Date:** 24.05.2026 13:04:42,57 
**Platform:** Windows 
**Test file:** 1 GB alternating 0x00/0x01 pattern 
**Integration:** Verlet (default) 
**Key level:** standard (5 bodies, 1M steps) 
 
| Mode | Operation | Time (s) | Throughput (GB/s) | Verify | 
|---|-----------|----------|-------------------|--------| 
| chaos | encrypt | 176118 | 0.000 | PASS | 
| chaos | decrypt | 176118 | 0.000 | PASS | 
| photon | encrypt | ERROR | 0 | FAIL | 
| photon | decrypt | ERROR | 0 | FAIL | 
| quantum | encrypt | ERROR | 0 | FAIL | 
| quantum | decrypt | ERROR | 0 | FAIL | 
|---|-----------|----------|-------------------|--------| 
 
*Benchmark completed at 24.05.2026 13:05:46,70* 
