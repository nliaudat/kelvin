#!/usr/bin/env bash
# =============================================================================
# run-kani.sh — Run Kani proofs inside Docker container
#
# Usage:
#   run-kani           — Run all 3 proofs with progress bar
#   run-kani --version — Print Kani version info
# =============================================================================
set -euo pipefail

KANI_ARGS=""
RESULTS_DIR="/kelvin/proofs/kani/results"

# ── Colors ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
CYAN='\033[0;36m'
YELLOW='\033[0;33m'
BOLD='\033[1m'
NC='\033[0m'

# ── Spinner ────────────────────────────────────────────────────────────────
SPINNER=('⠋' '⠙' '⠹' '⠸' '⠼' '⠴' '⠦' '⠧' '⠇' '⠏')
spinner_idx=0

next_spinner() {
    printf "%s" "${SPINNER[$spinner_idx]}"
    spinner_idx=$(( (spinner_idx + 1) % ${#SPINNER[@]} ))
}

# ── Progress bar: pipes Kani output, draws a live progress bar ─────────────
run_with_progress() {
    local harness_name="$1"
    local log_file="$2"
    local total_checks=0
    local done_checks=0
    local failed_checks=0
    local phase="compiling"
    local bar_len=30
    local last_line=""
    local kani_exit_code=0

    # Write full log via process substitution, parse progress from stdout
    # NOTE: Using process substitution (<(...)) instead of a pipe so the
    # while loop runs in the parent shell and variable modifications persist.
    exec 3>&1  # save stdout
    while IFS= read -r line; do
        last_line="$line"

        # Capture exit status sentinel appended by process substitution
        if [[ "$line" =~ ^EXIT_STATUS:([0-9]+) ]]; then
            kani_exit_code=${BASH_REMATCH[1]}

        # Detect compilation phase
        elif [[ "$line" == *"Compiling"* ]]; then
            phase="compiling"
            printf "\r  %s ${CYAN}Compiling...${NC}    \r" "$(next_spinner)" >&3

        elif [[ "$line" == *"Checking harness"* ]]; then
            phase="checking"
            printf "\r  %s ${CYAN}Checking harness...${NC}\r" "$(next_spinner)" >&3

        # Count total checks from GOTO generation
        # NOTE: Kani outputs "Generated N VCC(s)" once per verification round.
        # We only capture the first occurrence to avoid double-counting.
        elif [[ "$line" =~ Generated[[:space:]]([0-9]+)[[:space:]]VCC ]]; then
            if [ "$total_checks" -eq 0 ]; then
                total_checks=${BASH_REMATCH[1]}
            fi
            phase="running"

        elif [[ "$line" == *"Running propositional reduction"* ]]; then
            phase="propositional"
            printf "\r  %s ${YELLOW}Running propositional reduction...${NC}\r" "$(next_spinner)" >&3

        elif [[ "$line" == *"Solving with CaDiCaL"* ]]; then
            phase="solving"
            printf "\r  %s ${YELLOW}Solving...${NC}\r" "$(next_spinner)" >&3

        # Track individual check results
        elif [[ "$line" =~ ^Check[[:space:]]([0-9]+): ]]; then
            check_num=${BASH_REMATCH[1]}

        elif [[ "$line" == *"Status: SUCCESS"* ]]; then
            done_checks=$((done_checks + 1))
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "$phase" "" >&3

        elif [[ "$line" == *"Status: FAILURE"* ]]; then
            done_checks=$((done_checks + 1))
            failed_checks=$((failed_checks + 1))
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "$phase" "!" >&3

        # Summary lines
        elif [[ "$line" =~ SUMMARY:[[:space:]]([0-9]+)[[:space:]]of[[:space:]]([0-9]+)[[:space:]]failed ]]; then
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "done" "" >&3
            printf "\n" >&3

        elif [[ "$line" == "VERIFICATION:- SUCCESSFUL" ]]; then
            printf "\n  ${GREEN}✓ VERIFICATION: SUCCESSFUL${NC}\n" >&3

        elif [[ "$line" == "VERIFICATION:- FAILED" ]]; then
            printf "\n  ${RED}✗ VERIFICATION: FAILED${NC}\n" >&3
        fi
    done < <(cargo kani -p kelvin-core --harness "$harness_name" $KANI_ARGS 2>&1 | tee "$log_file"; echo "EXIT_STATUS:${PIPESTATUS[0]}")

    # If no checks were parsed (e.g. compile error), show final spinner
    if [ "$total_checks" -eq 0 ]; then
        if grep -iq "error" "$log_file" 2>/dev/null; then
            printf "\n  ${RED}✗ FAILED (compile or harness error)${NC}\n" >&3
        else
            printf "\n  ${GREEN}✓ Done${NC}\n" >&3
        fi
    fi

    return "$kani_exit_code"
}

# ── Draw progress bar ──────────────────────────────────────────────────────
draw_progress_bar() {
    local done=$1
    local total=$2
    local failed=$3
    local phase=$4
    local marker=$5
    local bar_len=30
    local pct=0
    local filled=0

    if [ "$total" -gt 0 ]; then
        pct=$((done * 100 / total))
        filled=$((done * bar_len / total))
    fi

    # Phase indicator
    local phase_char=""
    case "$phase" in
        compiling)     phase_char="${CYAN}C${NC}" ;;
        checking)      phase_char="${CYAN}?${NC}" ;;
        propositional) phase_char="${YELLOW}P${NC}" ;;
        solving)       phase_char="${YELLOW}S${NC}" ;;
        done)          phase_char="${GREEN}✓${NC}" ;;
        *)             phase_char=" " ;;
    esac

    # Build the bar
    local bar=""
    for ((i=0; i<filled; i++)); do
        bar="${bar}█"
    done
    for ((i=filled; i<bar_len; i++)); do
        bar="${bar}░"
    done

    # Color the bar based on failures
    local color="${GREEN}"
    if [ "$failed" -gt 0 ]; then
        color="${RED}"
    fi

    if [ "$done" -eq 0 ] && [ "$total" -eq 0 ]; then
        printf "\r  %s  ${phase_char}  ${CYAN}%s${NC}  [${color}]" "$(next_spinner)" "$phase" >&3
    else
        printf "\r  %s  [${color}%s${NC}]  %3d%%  ${done}/${total}  ${phase_char}" \
            "$(next_spinner)" "$bar" "$pct" >&3
    fi
}

# ── Print banner ───────────────────────────────────────────────────────────
print_banner() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║       KELVIN FORMAL VERIFICATION — Kani Model Checker       ║"
    echo "║       L0 Safety proofs in kelvin-core/src/fixed_math.rs     ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Kani:   $(cargo kani --version 2>/dev/null || echo 'unknown')"
    echo "  Rust:   $(rustc --version)"
    echo "  Host:   $(uname -a | awk '{print $1, $2, $3}')"
    echo "  Out:    $RESULTS_DIR/"
    echo ""
}

# ── Main ───────────────────────────────────────────────────────────────────
run_proofs() {
    print_banner
    mkdir -p "$RESULTS_DIR"

    TIMESTAMP=$(date --utc +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date -u +"%Y-%m-%dT%H:%M:%SZ")
    {
        echo "Kelvin Kani Verification Report"
        echo "================================"
        echo "Date:     $TIMESTAMP"
        echo "Kani:     $(cargo kani --version 2>/dev/null || echo 'unknown')"
        echo "Rust:     $(rustc --version)"
        echo "Host:     $(uname -a)"
        echo "================================"
    } > "$RESULTS_DIR/SUMMARY.txt"

    local all_passed=true
    local harnesses=(
        "verify_add_no_overflow:add never wraps in [-100, 100] AU"
        "verify_sub_no_overflow:sub never wraps in [-100, 100] AU"
        "verify_mul_range:mul range safety for [-4, 4] AU"
    )
    local total=${#harnesses[@]}
    local current=0

    for entry in "${harnesses[@]}"; do
        local name="${entry%%:*}"
        local desc="${entry#*:}"
        current=$((current + 1))

        local log_file="$RESULTS_DIR/${name}.log"

        echo ""
        echo "  ${BOLD}[$current/$total]${NC} ${CYAN}${name}${NC} — ${desc}"
        echo ""

        # Run with progress bar inline
        set +e
        run_with_progress "$name" "$log_file"
        local exit_code=$?
        set -e

        if [ $exit_code -ne 0 ]; then
            all_passed=false
            echo "  ${name}: ${RED}FAILED${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
        else
            # Check if log has VERIFICATION:- SUCCESSFUL
            if grep -q "VERIFICATION:- SUCCESSFUL" "$log_file" 2>/dev/null; then
                echo "  ${name}: ${GREEN}PASS${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
            elif grep -q "VERIFICATION:- FAILED" "$log_file" 2>/dev/null; then
                all_passed=false
                echo "  ${name}: ${RED}FAILED${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
            else
                # No verification line found — likely compile error
                if grep -qi "error" "$log_file" 2>/dev/null; then
                    all_passed=false
                    echo "  ${name}: ${RED}ERROR${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
                else
                    echo "  ${name}: ${YELLOW}UNKNOWN${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
                fi
            fi
        fi
    done

    echo ""
    if [ "$all_passed" = true ]; then
        echo "  ${GREEN}ALL PROOFS PASSED — ${total}/${total} harnesses verified${NC}" | tee -a "$RESULTS_DIR/SUMMARY.txt"
    else
        echo "  ${RED}SOME PROOFS FAILED — see $RESULTS_DIR/*.log${NC}" | tee -a "$RESULTS_DIR/SUMMARY.txt"
    fi
    echo ""
    echo "  Results saved to: $RESULTS_DIR/"
    echo ""

    echo "Note: proofs/kani/ contains template harnesses for additional levels"
    echo "(L1 functional equivalence, L2 composite correctness, L3 pipeline integrity)."
    echo "These must be integrated into kelvin-core/src/ for compilation."
    echo "See scripts/run_proofs.sh for the full harness listing."
    echo ""

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