# NIST SP 800-90B Entropy Source Validation

This directory contains tools for validating the Kelvin cryptosystem's keystream
against the NIST SP 800-90B entropy source standard, suitable for FIPS 140-3
submissions.

## Prerequisites

### 1. Install the NIST SP 800-90B Entropy Assessment Tool

```bash
git clone https://github.com/usnistgov/SP800-90B_EntropyAssessment.git
cd SP800-90B_EntropyAssessment
pip install -r requirements.txt
```

The tool requires Python 3. The main script is `ea_iid.py` (for IID data) and
`ea_non_iid.py` (for non-IID data). Since SHAKE256 output is expected to be IID,
use `ea_iid.py`.

### 2. Build the Kelvin keystream generator

```bash
cargo build --release -p nist_800_90b
```

## Step 1: Generate Raw Keystream

Generate 1 GB of raw keystream for analysis:

```bash
cargo run --release -p nist_800_90b -- generate \
    --size 1073741824 \
    --output keystream_1gb.bin
```

This uses the default 5-body orbital configuration with Euler integration
(maximum chaos amplification). The output is raw SHAKE256 XOR keystream from
`KelvinStreaming` (V2 mode) — the rawest form before any cipher layer.

### Options

| Flag | Default | Description |
|------|---------|-------------|
| `--size` | 1048576 (1 MB) | Number of bytes to generate |
| `--output` | `keystream.bin` | Output file path |
| `--config` | (default 5-body) | Custom orbital config JSON |
| `--verlet` | (Euler) | Use Verlet integration instead of Euler |

### Custom Configuration

To test with a custom orbital configuration:

```bash
cargo run --release -p kelvin-cli -- keygen --level standard --output my_config.json
cargo run --release -p nist_800_90b -- generate \
    --size 1073741824 \
    --output keystream_custom.bin \
    --config my_config.json
```

## Step 2: Run Built-in Health Tests

Before running the official NIST tool, verify the keystream passes the built-in
SP 800-90B health tests:

```bash
cargo run --release -p nist_800_90b -- analyze --input keystream_1gb.bin
```

This runs:
- Shannon Entropy (target: >7.5 bits/byte)
- Adjacent-byte Correlation (target: <0.01)
- Chi-square Byte Distribution (target: <310, df=255)
- Repetition Test (§4.4.1)
- Adaptive Proportion Test (§4.4.2)
- Runs Test (§2.3)
- Longest Run Test (§2.4)

## Step 3: Run Official NIST ea_iid Tool

```bash
cd SP800-90B_EntropyAssessment
python ea_iid.py -i ../keystream_1gb.bin -o results_ea_iid.txt
```

The `ea_iid.py` tool will:
1. Estimate the min-entropy of the raw keystream
2. Apply the SP 800-90B restart tests
3. Produce a comprehensive report

### Interpreting Results

The key metric is **min-entropy per sample** (per byte). For a perfect random
source, this should be close to 8.0 bits/byte. NIST SP 800-90B requires:

- **H_IID**: Min-entropy estimate from the IID track
- **H_non-IID**: Min-entropy estimate from the non-IID track (if applicable)
- **H_min**: The final min-entropy estimate (the lower of the two)

For SHAKE256-conditioned output, expect H_min > 7.9 bits/byte.

## Step 4: Document Conditioning Component (NIST SP 800-90C)

Per NIST SP 800-90C, the conditioning component must be documented:

| Property | Value |
|----------|-------|
| **Conditioning function** | SHAKE256 (XOF) |
| **NIST standard** | FIPS 202 / SP 800-185 |
| **Input** | Orbital state (positions, velocities, masses, accelerations, G, softening, step counter, domain separator) |
| **Output length** | Configurable (default: 64 KB per step) |
| **Domain separation** | `b"kelvin-streaming-v2-v1-000000000"` |
| **Security strength** | 256 bits (classical) / 128 bits (quantum) |

The raw noise source is the **n-body gravitational simulation** (Verlet or Euler
integration). The conditioning is SHAKE256, which is a NIST-approved conditioning
function per SP 800-90C.

## Step 5: Generate FIPS 140-3 Report

Use the template at `documentation/nist_800_90b_report.md` to compile the final
report. Fill in the results from `ea_iid.py` and the built-in health tests.

## Quick Reference

```bash
# Full workflow
cargo build --release -p nist_800_90b
cargo run --release -p nist_800_90b -- generate --size 1073741824 --output keystream_1gb.bin
cargo run --release -p nist_800_90b -- analyze --input keystream_1gb.bin
python ea_iid.py -i keystream_1gb.bin -o results_ea_iid.txt

# Quick test (1 MB, fast)
cargo run --release -p nist_800_90b -- generate --size 1048576 --output keystream_1mb.bin
cargo run --release -p nist_800_90b -- analyze --input keystream_1mb.bin
```
