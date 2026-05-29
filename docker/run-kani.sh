#!/usr/bin/env bash
# =============================================================================
# run-kani.sh — Run Kani proofs inside Docker container
#
# This is the Docker entrypoint for formal verification. It runs the 5 L0
# Safety harnesses in kelvin-core/src/fixed_math.rs (the only harnesses
# currently compiled into the crate).
#
# Usage:
#   run-kani           — Run all 5 proofs (full verification)
#   run-kani --fast    — Same as default (only 5 harnesses exist)
#   run-kani --version — Print Kani version info
# =============================================================================
set -euo pipefail

KANI_ARGS=""

# Results directory — colocated with proof harnesses in proofs/kani/
RESULTS_DIR="/kelvin/proofs/kani/results"

print_banner() {
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║       KELVIN FORMAL VERIFICATION — Kani Model Checker       ║"
    echo "║       L0 Safety proofs in kelvin-core/src/fixed_math.rs     ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Kani version: $(cargo kani --version 2>/dev/null || echo 'unknown')"
    echo "Rust version:  $(rustc --version)"
    echo "Platform:      $(uname -a)"
    echo ""
    echo "Available harnesses (5 L0 Safety):"
    echo "  verify_add_no_overflow"
    echo "  verify_sub_no_overflow"
    echo "  verify_mul_no_overflow"
    echo "  verify_div_no_panic"
    echo "  verify_sqrt_bounded"
    echo ""
    echo "Results saved to: $RESULTS_DIR/"
    echo ""
}

# Run a single harness and tee output to a log file
run_harness() {
    local name="$1"
    local desc="$2"
    local log_file="$RESULTS_DIR/${name}.log"

    echo "[$3/5] $name — $desc"
    cargo kani -p kelvin-core --harness "$name" $KANI_ARGS 2>&1 | tee "$log_file"

    # Check exit code — return it so the script can fail
    return ${PIPESTATUS[0]}
}

run_proofs() {
    print_banner

    # Create results directory inside the mounted workspace (persists across runs)
    mkdir -p "$RESULTS_DIR"

    # Save a summary header with timestamp
    TIMESTAMP=$(date --utc +"%Y-%m-%dT%H:%M:%SZ") || TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    SUMMARY_FILE="$RESULTS_DIR/SUMMARY.txt"
    {
        echo "Kelvin Kani Verification Report"
        echo "================================"
        echo "Date:       $TIMESTAMP"
        echo "Kani:       $(cargo kani --version 2>/dev/null || echo 'unknown')"
        echo "Rust:       $(rustc --version)"
        echo "Platform:   $(uname -a)"
        echo "================================"
        echo ""
    } | tee "$SUMMARY_FILE"

    local all_passed=true
    local i=1

    run_harness "verify_add_no_overflow"   "add never wraps in [-100, 100] AU" "$i" || all_passed=false && i=$((i + 1))
    run_harness "verify_sub_no_overflow"   "sub never wraps in [-100, 100] AU" "$i" || all_passed=false && i=$((i + 1))
    run_harness "verify_mul_no_overflow"   "mul splitting safe for [-100, 100] AU" "$i" || all_passed=false && i=$((i + 1))
    run_harness "verify_div_no_panic"      "div never panics for G / bounded dist³" "$i" || all_passed=false && i=$((i + 1))
    run_harness "verify_sqrt_bounded"      "sqrt safe for squared distances up to (200 AU)²" "$i" || all_passed=false && i=$((i + 1))

    echo ""
    if [ "$all_passed" = true ]; then
        echo "ALL PROOFS PASSED — 5/5 harnesses verified" | tee -a "$SUMMARY_FILE"
        echo ""
        echo "Results saved to: $RESULTS_DIR/"
    else
        echo "SOME PROOFS FAILED — see logs above" | tee -a "$SUMMARY_FILE"
        echo ""
        echo "Individual log files: $RESULTS_DIR/*.log"
    fi
    echo ""

    echo "Note: proofs/kani/ contains template harnesses for additional levels"
    echo "(L1 functional equivalence, L2 composite correctness, L3 pipeline integrity)."
    echo "These must be integrated into kelvin-core/src/ for compilation."
    echo "See scripts/run_proofs.sh for the full harness listing."
    echo ""

    # Exit with proper code so `run_proofs --exit` can be used in CI
    if [ "${1:-}" = "--exit" ]; then
        [ "$all_passed" = true ] && exit 0 || exit 1
    fi
}

case "${1:-}" in
    --version)
        cargo kani --version
        ;;
    *)
        run_proofs
        ;;
esac