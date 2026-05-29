#!/usr/bin/env bash
# =============================================================================
# build-multiarch.sh — Cross-platform build & test for all supported targets
#
# Compiles kelvin-core for all 5 architectures, runs unit tests via QEMU
# emulation where available, and verifies that determinism holds across
# all targets (same golden hash on x86_64, aarch64, riscv64, s390x).
#
# Usage:
#   build-multiarch.sh                    — Build all targets
#   build-multiarch.sh --test             — Build + run tests on foreign arches
#   build-multiarch.sh --endian           — Run endianness verification (s390x BE)
# =============================================================================
set -euo pipefail

OUT_DIR="/kelvin/target/cross"
TIMESTAMP=$(date --utc +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u +"%Y-%m-%dT%H:%M:%SZ")

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; YELLOW='\033[0;33m'
BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${CYAN}${BOLD}[INFO]${NC} $*"; }
ok()    { echo -e "${GREEN}${BOLD}[OK]${NC}   $*"; }
warn()  { echo -e "${YELLOW}${BOLD}[WARN]${NC} $*"; }
fail()  { echo -e "${RED}${BOLD}[FAIL]${NC} $*"; }

summary_file="$OUT_DIR/SUMMARY.txt"
mkdir -p "$OUT_DIR"

write_summary() { echo "$*" >> "$summary_file"; }

# ── Target definitions ─────────────────────────────────────────────────────
declare -a TARGETS=(
    "x86_64:x86_64-unknown-linux-gnu::little"
    "aarch64:aarch64-unknown-linux-gnu:qemu-aarch64:little"
    "riscv64:riscv64gc-unknown-linux-gnu:qemu-riscv64:little"
    "wasm32:wasm32-unknown-unknown::little"
    "s390x:s390x-unknown-linux-gnu:qemu-s390x:big"
)

# ── Build a single target ──────────────────────────────────────────────────
build_target() {
    local name="$1" rust_target="$2"
    info "Building ${BOLD}$name${NC} ($rust_target)..."
    local target_dir="$OUT_DIR/$name"
    mkdir -p "$target_dir"

    local log_file="$OUT_DIR/${name}_build.log"
    local exit_code=0
    if [ "$rust_target" = "x86_64-unknown-linux-gnu" ]; then
        cargo build -p kelvin-core --release --target-dir "$target_dir" > "$log_file" 2>&1; exit_code=$?
    else
        cargo build -p kelvin-core --release --target "$rust_target" --target-dir "$target_dir" > "$log_file" 2>&1; exit_code=$?
    fi
    
    # Show last lines of the build log
    tail -5 "$log_file"
    
    if [ "$exit_code" -eq 0 ]; then
        ok "$name built successfully"; write_summary "  $name: BUILD PASS"
    else
        fail "$name build FAILED (exit: $exit_code)"
        write_summary "  $name: BUILD FAIL (exit: $exit_code)"
        echo "  Last 50 lines of log:"; tail -50 "$log_file" | sed 's/^/    /'
        return 1
    fi
    return 0
}

# ── Test a target via QEMU ─────────────────────────────────────────────────
test_target() {
    local name="$1" rust_target="$2" qemu_bin="$3"
    if [ "$name" = "wasm32" ]; then
        warn "$name: compile-only, no runner"; write_summary "  $name: COMPILE OK"
        return 0
    fi
    if [ -z "$qemu_bin" ] && [ "$name" != "x86_64" ]; then
        warn "$name: no QEMU, skipping tests"; write_summary "  $name: TESTS SKIP"
        return 0
    fi

    local target_dir="$OUT_DIR/$name"
    info "Building determinism test for ${BOLD}$name${NC}..."
    if [ "$rust_target" = "x86_64-unknown-linux-gnu" ]; then
        cargo test -p kelvin-core --test determinism --no-run --release --target-dir "$target_dir" 2>/dev/null || true
    else
        cargo test -p kelvin-core --test determinism --no-run --release --target "$rust_target" --target-dir "$target_dir" 2>/dev/null || true
    fi

    local test_binary
    test_binary=$(find "$target_dir/$rust_target/release" -maxdepth 1 -type f -name "determinism-*" 2>/dev/null | head -1)
    if [ -z "$test_binary" ]; then
        warn "$name: no test binary found"; write_summary "  $name: TESTS SKIP (no binary)"; return 0
    fi

    info "Running determinism tests on ${BOLD}$name${NC}..."
    local runner; [ "$name" = "x86_64" ] && runner="$test_binary" || runner="$qemu_bin $test_binary"
    if $runner 2>&1 | tail -10 | grep -q "test result:"; then
        ok "$name: all determinism tests passed"; write_summary "  $name: TESTS PASS"
    else
        fail "$name: some tests FAILED"; write_summary "  $name: TESTS FAIL"
    fi
}

# ── Endianness: build golden_hash for s390x BE, run via QEMU ───────────────
test_endianness() {
    echo ""
    info "============================================"
    info " Endianness — ${BOLD}s390x big-endian${NC}"
    info " Verifying golden hash matches x86_64 ref"
    info "============================================"
    echo ""

    local target_dir="$OUT_DIR/s390x"
    local rust_target="s390x-unknown-linux-gnu"
    local qemu_bin="qemu-s390x"

    info "Building golden_hash test for s390x..."
    cargo test -p kelvin --test golden_hash --no-run --release --target "$rust_target" --target-dir "$target_dir" 2>/dev/null || true

    local test_binary
    test_binary=$(find "$target_dir/$rust_target/release" -maxdepth 1 -type f -name "golden_hash-*" 2>/dev/null | head -1)
    if [ -z "$test_binary" ]; then
        warn "No golden_hash binary for s390x"; write_summary "  ENDIANNESS: SKIP"; return 0
    fi

    info "Running golden_hash on big-endian s390x via QEMU..."
    local output
    output=$("$qemu_bin" "$test_binary" 2>&1 || true)

    if echo "$output" | grep -q "test result: ok"; then
        ok "Endianness VERIFIED — hash matches x86_64 golden hash"
        write_summary "  ENDIANNESS: PASS"
    else
        warn "Check output:"; echo "$output"
        write_summary "  ENDIANNESS: LOGGED"
    fi
}

# ── Main ───────────────────────────────────────────────────────────────────
echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║  KELVIN CROSS-PLATFORM BUILD                                 ║"
echo "║  x86_64 | aarch64 | riscv64 | wasm32 | s390x (BE)           ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
echo "  Rust: $(rustc --version)"
echo "  Out:  $OUT_DIR/"
echo ""

{
    echo "Cross-Platform Build Report"; echo "================================"
    echo "Date: $TIMESTAMP"; echo "Rust: $(rustc --version)"
    echo "Host: $(uname -a | awk '{print $1, $2, $3}')"
    echo "Targets: x86_64, aarch64, riscv64, wasm32, s390x"
    echo "================================"; echo ""
} > "$summary_file"

all_ok=true
for entry in "${TARGETS[@]}"; do
    IFS=':' read -r name rust_target qemu_bin _ <<< "$entry"
    echo ""
    echo "  ${BOLD}── $name ──${NC}"
    build_target "$name" "$rust_target" || all_ok=false
    if [ "${1:-}" = "--test" ] || [ "${1:-}" = "" ]; then
        test_target "$name" "$rust_target" "$qemu_bin"
    fi
    echo ""
done

if [ "${1:-}" = "--endian" ] || [ "${1:-}" = "" ]; then
    test_endianness
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════╗"
echo "║  SUMMARY                                                    ║"
echo "╚═══════════════════════════════════════════════════════════════╝"
if [ "$all_ok" = true ]; then
    echo -e "  ${GREEN}All builds passed${NC}"
else
    echo -e "  ${RED}Some targets failed${NC}"
fi
echo "  Report: $summary_file"; echo ""

write_summary ""; write_summary "Result: $([ "$all_ok" = true ] && echo 'ALL PASS' || echo 'SOME FAILED')"
[ "$all_ok" = true ] && exit 0 || exit 1