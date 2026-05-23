#!/usr/bin/env bash
# ==============================================================================
# Kelvin -- 100 GB Encryption Benchmark (Unix Shell)
# ==============================================================================
# Benchmarks all 4 modes (secure, chaos, photon, quantum) on a 100 GB file
# with alternating 0x00/0x01 pattern. Outputs results to documentation/100gb_benchmark.md
#
# Usage:
#   scripts/bench-100gb.sh
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Change to workspace root
cd "$(dirname "$0")/.."

TEMP_DIR="${TEMP_DIR:-/tmp/kelvin_temp}"
KEY_FILE="${TEMP_DIR}/key.json"
INPUT_FILE="${TEMP_DIR}/test_100gb.bin"
ENC_FILE="${TEMP_DIR}/test_100gb.enc"
DEC_FILE="${TEMP_DIR}/test_100gb_dec.bin"
REPORT="documentation/100gb_benchmark.md"
FILE_SIZE=107374182400
BUFFER_SIZE=1048576

echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Kelvin 100 GB Encryption Benchmark${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# ---------------------------------------------------------------------------
# 1. Setup
# ---------------------------------------------------------------------------
echo -e "${CYAN}[1/6] Creating temp directory...${NC}"
mkdir -p "${TEMP_DIR}"

echo -e "${CYAN}[2/6] Building release CLI...${NC}"
cargo build --release -p kelvin-cli
echo -e "${GREEN}  Done${NC}"

echo -e "${CYAN}[3/6] Generating orbital config...${NC}"
target/release/kelvin keygen --level standard --output "${KEY_FILE}"
echo -e "${GREEN}  Done${NC}"

echo -e "${CYAN}[4/6] Generating 100 GB alternating pattern file...${NC}"
echo -e "${YELLOW}  This may take a few minutes...${NC}"

# Generate alternating 0x00/0x01 pattern using dd with a pattern buffer
python3 -c "
import sys
buf = bytearray(${BUFFER_SIZE})
for i in range(0, ${BUFFER_SIZE}, 2):
    buf[i] = 0x00
    if i + 1 < ${BUFFER_SIZE}:
        buf[i+1] = 0x01
total = ${FILE_SIZE}
written = 0
with open('${INPUT_FILE}', 'wb') as f:
    while written < total:
        chunk = buf[:min(len(buf), total - written)]
        f.write(chunk)
        written += len(chunk)
        if written % (10 * ${BUFFER_SIZE}) == 0:
            sys.stderr.write('.')
            sys.stderr.flush()
sys.stderr.write('\n')
"
echo -e "${GREEN}  Done${NC}"

echo -e "${CYAN}[5/6] Computing input file hash...${NC}"
INPUT_HASH=$(sha256sum "${INPUT_FILE}" | cut -d' ' -f1)
echo -e "${GREEN}  SHA256: ${INPUT_HASH}${NC}"

# ---------------------------------------------------------------------------
# 2. Benchmark
# ---------------------------------------------------------------------------
echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  Running Benchmarks${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# Initialize report
cat > "${REPORT}" << EOF
# Kelvin 100 GB Encryption Benchmark

**Date:** $(date)
**Platform:** $(uname -srm)
**Test file:** 100 GB alternating 0x00/0x01 pattern
**Integration:** Euler (default)
**Key level:** standard (5 bodies, 1M steps)

| Mode | Operation | Time (s) | Throughput (GB/s) | Verify |
|------|-----------|----------|-------------------|--------|
EOF

for mode in secure chaos photon quantum; do
    echo ""
    echo -e "${CYAN}--- Mode: ${mode} ---${NC}"

    # Encrypt
    echo -e "${YELLOW}  Encrypting...${NC}"
    ENC_START=$(date +%s.%N)
    if target/release/kelvin encrypt --mode "${mode}" --config "${KEY_FILE}" --input "${INPUT_FILE}" --output "${ENC_FILE}"; then
        ENC_END=$(date +%s.%N)
        ENC_TIME=$(python3 -c "print(f'{${ENC_END} - ${ENC_START}:.3f}')")
        echo -e "${GREEN}  Encrypt time: ${ENC_TIME} s${NC}"
    else
        ENC_TIME="ERROR"
        echo -e "${RED}  Encryption failed for ${mode}${NC}"
    fi

    # Decrypt
    echo -e "${YELLOW}  Decrypting...${NC}"
    DEC_START=$(date +%s.%N)
    if target/release/kelvin decrypt --mode "${mode}" --config "${KEY_FILE}" --input "${ENC_FILE}" --output "${DEC_FILE}"; then
        DEC_END=$(date +%s.%N)
        DEC_TIME=$(python3 -c "print(f'{${DEC_END} - ${DEC_START}:.3f}')")
        echo -e "${GREEN}  Decrypt time: ${DEC_TIME} s${NC}"
    else
        DEC_TIME="ERROR"
        echo -e "${RED}  Decryption failed for ${mode}${NC}"
    fi

    # Verify
    DEC_HASH=$(sha256sum "${DEC_FILE}" | cut -d' ' -f1)
    if [ "${DEC_HASH}" = "${INPUT_HASH}" ]; then
        VERIFY="✅"
        echo -e "${GREEN}  SHA256 match: ✅${NC}"
    else
        VERIFY="❌"
        echo -e "${RED}  SHA256 MISMATCH!${NC}"
    fi

    # Compute throughput
    if [ "${ENC_TIME}" != "ERROR" ]; then
        ENC_GBPS=$(python3 -c "print(f'{100.0 / ${ENC_TIME}:.2f}')")
    else
        ENC_GBPS="0"
    fi
    if [ "${DEC_TIME}" != "ERROR" ]; then
        DEC_GBPS=$(python3 -c "print(f'{100.0 / ${DEC_TIME}:.2f}')")
    else
        DEC_GBPS="0"
    fi

    # Append to report
    echo "| ${mode} | encrypt | ${ENC_TIME} | ${ENC_GBPS} | ${VERIFY} |" >> "${REPORT}"
    echo "| ${mode} | decrypt | ${DEC_TIME} | ${DEC_GBPS} | ${VERIFY} |" >> "${REPORT}"

    # Clean up intermediate files for this mode
    rm -f "${ENC_FILE}" "${DEC_FILE}"
done

# Close report
echo "|------|-----------|----------|-------------------|--------|" >> "${REPORT}"
echo "" >> "${REPORT}"
echo "*Benchmark completed at $(date)*" >> "${REPORT}"

# ---------------------------------------------------------------------------
# 3. Cleanup
# ---------------------------------------------------------------------------
echo ""
echo -e "${CYAN}[6/6] Cleaning up...${NC}"
rm -f "${INPUT_FILE}" "${KEY_FILE}"
rmdir "${TEMP_DIR}" 2>/dev/null || true

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  Benchmark Complete${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Report written to: ${REPORT}"
cat "${REPORT}"

exit 0
