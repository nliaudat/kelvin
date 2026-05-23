#!/usr/bin/env bash
# ==============================================================================
# build-ea-iid.sh — Build NIST SP 800-90B Entropy Assessment Tool (Linux/macOS)
# ==============================================================================
# Compiles the official NIST entropy assessment tool (ea_iid) from the
# tests/ea_iid/ git submodule using Docker.
#
# Requires: Docker
#
# Usage:
#   ./tests/build-ea-iid.sh
#
# Exit code: 0 on success, 1 on failure
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
NC='\033[0m'

echo ""
echo -e "${CYAN}========================================${NC}"
echo -e "${CYAN}  NIST SP 800-90B ea_iid Build Script${NC}"
echo -e "${CYAN}========================================${NC}"
echo ""

# ---------------------------------------------------------------------------
# 1. Initialize submodule
# ---------------------------------------------------------------------------
echo "[1/4] Initializing git submodule..."
git submodule update --init tests/ea_iid
echo -e "${GREEN}OK${NC}"

# ---------------------------------------------------------------------------
# 2. Check for Docker
# ---------------------------------------------------------------------------
echo "[2/4] Checking for Docker..."
if ! command -v docker &>/dev/null; then
    echo -e "${RED}FAILED: Docker not found. Install Docker from:${NC}"
    echo "  https://docs.docker.com/get-docker/"
    exit 1
fi
echo -e "${GREEN}Found Docker${NC}"

# ---------------------------------------------------------------------------
# 3. Build with Docker
# ---------------------------------------------------------------------------
echo "[3/4] Building ea_iid in Docker..."
echo "This will download an Ubuntu base image (~80 MB) on first run."

docker build -t ea_iid-builder -f tests/ea_iid/Dockerfile tests/ea_iid
echo -e "${GREEN}Docker build successful${NC}"

# ---------------------------------------------------------------------------
# 4. Extract binaries
# ---------------------------------------------------------------------------
echo "[4/4] Extracting binaries..."

# Create a temporary container and copy binaries out
CONTAINER_ID=$(docker create ea_iid-builder /bin/sh)
docker cp "${CONTAINER_ID}:/ea_iid" tests/ea_iid/ea_iid
docker cp "${CONTAINER_ID}:/ea_non_iid" tests/ea_iid/ea_non_iid
docker cp "${CONTAINER_ID}:/ea_restart" tests/ea_iid/ea_restart
docker cp "${CONTAINER_ID}:/ea_conditioning" tests/ea_iid/ea_conditioning
docker cp "${CONTAINER_ID}:/ea_transpose" tests/ea_iid/ea_transpose
docker rm "${CONTAINER_ID}" > /dev/null

echo -e "${GREEN}Binaries extracted to tests/ea_iid/${NC}"

# ---------------------------------------------------------------------------
# Done
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  BUILD SUCCESSFUL${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "Binaries:"
echo "  tests/ea_iid/ea_iid              — IID tests"
echo "  tests/ea_iid/ea_non_iid          — Non-IID tests"
echo "  tests/ea_iid/ea_restart          — Restart tests"
echo "  tests/ea_iid/ea_conditioning     — Conditioning calculator"
echo "  tests/ea_iid/ea_transpose        — Data transpose utility"
echo ""
echo "To run on Kelvin keystream:"
echo "  python tests/ea_iid/ea_iid.py -i keystream_1gb.bin -o results.txt"
echo ""
