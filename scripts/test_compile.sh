#!/usr/bin/env bash
# ==============================================================================
# Kelvin — Compile & Lint Tests (Linux / macOS)
# ==============================================================================
# Verifies that all workspace members compile and pass lint checks.
#
# Exit code is 0 only if ALL steps pass.
#
# Usage:
#   ./scripts/test_compile.sh
# ==============================================================================

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

step() {
    echo ""
    echo -e "${CYAN}========================================${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}========================================${NC}"
}

pass() {
    echo -e "${GREEN}PASSED${NC}"
}

# Initialize git submodules (NIST SP 800-90B tools)
if [ -f .gitmodules ]; then
    git submodule update --init --recursive
fi

step "1/5: Build workspace (default features)"
cargo build --workspace
pass

step "2/5: Build with AES-NI feature"
cargo build -p kelvin-stream --features aes-ni
cargo build -p kelvin --features aes-ni
pass

step "3/5: Build V2 Streaming example"
cargo build --example simple_streaming -p kelvin
pass

step "4/5: Build kelvin-ffi (C FFI bindings)"
cargo build -p kelvin-ffi
pass

step "5/5: Clippy (deny warnings)"
cargo clippy --workspace -- -D warnings
pass

# ---------------------------------------------------------------------------
# All passed
# ---------------------------------------------------------------------------
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  ALL COMPILE TESTS PASSED${NC}"
echo -e "${GREEN}========================================${NC}"
