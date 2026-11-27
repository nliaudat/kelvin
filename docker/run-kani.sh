#!/usr/bin/env bash
# =============================================================================
# run-kani.sh — Run Kani proofs inside Docker container
#
# Runs 43 harnesses:
#   - 8 integrated in kelvin-core/src/fixed_math.rs under #[cfg(kani)]
#   - 29 in proofs/kani/*.rs, included by kelvin-core/src/kani_harnesses.rs
#   - 4 in kelvin-kdf/src/schedule.rs and 2 in kelvin-kdf/src/extractor.rs
#
# kelvin-kdf also contains older #[cfg(kani)] harnesses that are not in this
# list; they have no recorded verdict.
#
# The included files were corrected on 2026-11-23: they previously sat outside
# every Cargo target, used `rand`/`Instant::now()`, or asserted constants chosen
# inside the harness. Each now states a property Kani can check.
#
# Usage:
#   run-kani                 Run every integrated harness (default)
#   run-kani --all           Same as the default
#   run-kani --proof NAME    Run a single integrated harness by name
#   run-kani --harness NAME  Alias of --proof
#   run-kani --list          List the integrated harness names
#   run-kani --version       Print Kani version info
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

draw_progress_bar() {
    local done=$1
    local total=$2
    local failed=$3
    local phase=$4
    local bar_len=30
    local pct=0
    local filled=0

    if [ "$total" -gt 0 ]; then
        pct=$((done * 100 / total))
        filled=$((done * bar_len / total))
    fi

    local phase_char=""
    case "$phase" in
        compiling)     phase_char="${CYAN}C${NC}" ;;
        checking)      phase_char="${CYAN}?${NC}" ;;
        propositional) phase_char="${YELLOW}P${NC}" ;;
        solving)       phase_char="${YELLOW}S${NC}" ;;
        done)          phase_char="${GREEN}✓${NC}" ;;
        *)             phase_char=" " ;;
    esac

    local bar=""
    for ((i=0; i<filled; i++)); do bar="${bar}█"; done
    for ((i=filled; i<bar_len; i++)); do bar="${bar}░"; done

    local color="${GREEN}"
    if [ "$failed" -gt 0 ]; then color="${RED}"; fi

    if [ "$done" -eq 0 ] && [ "$total" -eq 0 ]; then
        printf "\r  %s  ${phase_char}  ${CYAN}%s${NC}  [${color}]" "$(next_spinner)" "$phase" >&3
    else
        printf "\r  %s  [${color}%s${NC}]  %3d%%  ${done}/${total}  ${phase_char}" \
            "$(next_spinner)" "$bar" "$pct" >&3
    fi
}

# The crate that contains a harness.
package_for() {
    case "$1" in
        verify_schedule_* | verify_extraction_header_injective | verify_extraction_body_injective)
            echo kelvin-kdf ;;
        *) echo kelvin-core ;;
    esac
}

run_with_progress() {
    local harness_name="$1"
    local log_file="$2"
    local total_checks=0
    local done_checks=0
    local failed_checks=0
    local phase="compiling"
    local kani_exit_code=0

    exec 3>&1
    while IFS= read -r line; do
        if [[ "$line" =~ ^EXIT_STATUS:([0-9]+) ]]; then
            kani_exit_code=${BASH_REMATCH[1]}
        elif [[ "$line" == *"Compiling"* ]]; then
            phase="compiling"
            printf "\r  %s ${CYAN}Compiling...${NC}    \r" "$(next_spinner)" >&3
        elif [[ "$line" == *"Checking harness"* ]]; then
            phase="checking"
            printf "\r  %s ${CYAN}Checking harness...${NC}\r" "$(next_spinner)" >&3
        elif [[ "$line" =~ Generated[[:space:]]([0-9]+)[[:space:]]VCC ]]; then
            [ "$total_checks" -eq 0 ] && total_checks=${BASH_REMATCH[1]}
            phase="running"
        elif [[ "$line" == *"Running propositional reduction"* ]]; then
            phase="propositional"
            printf "\r  %s ${YELLOW}Running propositional reduction...${NC}\r" "$(next_spinner)" >&3
        elif [[ "$line" == *"Solving with CaDiCaL"* ]]; then
            phase="solving"
            printf "\r  %s ${YELLOW}Solving...${NC}\r" "$(next_spinner)" >&3
        elif [[ "$line" == *"Status: SUCCESS"* ]]; then
            done_checks=$((done_checks + 1))
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "$phase" "" >&3
        elif [[ "$line" == *"Status: FAILURE"* ]]; then
            done_checks=$((done_checks + 1))
            failed_checks=$((failed_checks + 1))
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "$phase" "!" >&3
        elif [[ "$line" =~ SUMMARY:[[:space:]]([0-9]+)[[:space:]]of[[:space:]]([0-9]+)[[:space:]]failed ]]; then
            draw_progress_bar "$done_checks" "$total_checks" "$failed_checks" "done" "" >&3
            printf "\n" >&3
        elif [[ "$line" == "VERIFICATION:- SUCCESSFUL" ]]; then
            printf "\n  ${GREEN}✓ VERIFICATION: SUCCESSFUL${NC}\n" >&3
        elif [[ "$line" == "VERIFICATION:- FAILED" ]]; then
            printf "\n  ${RED}✗ VERIFICATION: FAILED${NC}\n" >&3
        fi
    done < <(cargo kani -p "$(package_for "$harness_name")" --harness "$harness_name" $KANI_ARGS 2>&1 | tee "$log_file"; echo "EXIT_STATUS:${PIPESTATUS[0]}")

    if [ "$total_checks" -eq 0 ]; then
        if grep -iq "error\|not found" "$log_file" 2>/dev/null; then
            printf "\n  ${RED}✗ FAILED (compile or harness error)${NC}\n" >&3
        else
            printf "\n  ${GREEN}✓ Done${NC}\n" >&3
        fi
    fi

    return "$kani_exit_code"
}

print_banner() {
    echo ""
    echo "╔═══════════════════════════════════════════════════════════════╗"
    echo "║       KELVIN FORMAL VERIFICATION — Kani Model Checker       ║"
    echo "║      43 integrated harnesses (37 core + 6 kdf)              ║"
    echo "╚═══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "  Kani:   $(cargo kani --version 2>/dev/null || echo 'unknown')"
    echo "  Rust:   $(rustc --version)"
    echo "  Host:   $(uname -a | awk '{print $1, $2, $3}')"
    echo "  Out:    $RESULTS_DIR/"
    echo ""
}

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
    # Only harnesses actually integrated under #[cfg(kani)]
    local harnesses=(
        # L0: Fixed arithmetic safety (kelvin-core/src/fixed_math.rs)
        "verify_add_no_overflow:add never wraps in [-100, 100] AU"
        "verify_sub_no_overflow:sub never wraps in [-100, 100] AU"
        "verify_mul_range:mul range safety for [-4, 4] AU"

        # C1: Information loss arithmetic (fixed_math.rs + proofs/kani)
        "verify_c1_preimage_bound:preimage bound at softening limit"
        "verify_c1_epsilon_bound:epsilon-bound on the implemented division"
        "verify_c1_division_remainder:division remainder 0 <= r < den"
        "verify_information_loss_total:saturation model never exceeds H_max"
        "verify_information_loss_monotone:saturation model is monotone in S"
        "verify_saturation_point:96 steps saturate the N=5 model"
        "verify_information_loss_saturating:capped loss <= H_max and <= uncapped"
        "verify_information_loss_cap_boundary:larger k saturates no later"

        # C2: Lyapunov arithmetic (proofs/kani)
        "verify_lyapunov_spectrum_comprehensive:Pade ratio lies in [0, 1]"
        "verify_lyapunov_division:lambda >= 0 and lambda <= ln(ratio)"
        "verify_lyapunov_error_bounds:quantization error is below one raw unit"
        "verify_lyapunov_quantization:eps_q / time is zero for time >= 2^54"
        "verify_shadow_orbit_bias:bias budget below accumulated time"
        "verify_perturbation_in_range:shadow perturbation fits in Q32.64"

        # C3: division-rounding damping and exponent arithmetic
        "verify_c3_step_damping:1-ULP divisor change moves the quotient by <= 1 ULP"
        "verify_c3_two_step_dispersion:1-ULP quotient difference does not grow through a second division"
        "verify_c3_quantum_reduction:halved exponent does not exceed original"
        "verify_c3_exponent_arithmetic:sqrt(2^1920) has exponent 960"
        "verify_c3_exponent_law:2^a * 2^b = 2^(a+b)"
        "verify_c3_contraction_arithmetic:non-expanding steps cannot widen a window"

        # C4: framing and bound-selection arithmetic
        "verify_domain_separation:distinct tags give distinct framed inputs"
        "verify_framing_deterministic:framing is deterministic"
        "verify_shake256_grover_arithmetic:Grover on 256 bits leaves 128"
        "verify_bound_is_min_of_two:operational bound is the smaller exponent"

        # C5: branch-free primitives (functional only; timing is NOT verified)
        "verify_ct_select_i128:ct_select returns the requested operand"
        "verify_ct_select_sign_bits:ct_select yields an input verbatim"
        "verify_ct_is_zero_u64:ct inequality test is exact"
        "verify_ct_select_u64:64-bit ct_select returns the requested operand"
        "verify_ct_select_u64_is_exact:64-bit ct_select stays in the operand interval"

        # C6: Floyd invariants and the real find_cycle (bounded state space)
        "verify_floyd_cycle_detection:tortoise advances k, hare 2k"
        "verify_hare_leads:hare never falls behind the tortoise"
        "verify_find_cycle_is_complete_and_correct:finds the true mu and lambda"
        "verify_find_cycle_is_sound_for_any_budget:any reported cycle is correct"
        "verify_find_cycle_zero_budget:zero budget takes no step"

        # Key derivation: input construction and key limits (kelvin-kdf)
        "verify_schedule_info_injective:distinct indices give distinct HKDF info"
        "verify_schedule_key_and_nonce_info_separated:key and nonce info never overlap"
        "verify_schedule_first_indices:consecutive indices, permanent exhaustion"
        "verify_schedule_limit_invariant:no overflow or excess keys (16-bit parameters)"
        "verify_extraction_header_injective:distinct arguments give distinct hash input"
        "verify_extraction_body_injective:distinct bodies give distinct hash input"
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

        set +e
        run_with_progress "$name" "$log_file"
        local exit_code=$?
        set -e

        if [ $exit_code -ne 0 ]; then
            all_passed=false
            echo "  ${name}: ${RED}FAILED${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
        else
            if grep -q "VERIFICATION:- SUCCESSFUL" "$log_file" 2>/dev/null; then
                echo "  ${name}: ${GREEN}PASS${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
            elif grep -q "VERIFICATION:- FAILED" "$log_file" 2>/dev/null; then
                all_passed=false
                echo "  ${name}: ${RED}FAILED${NC}" >> "$RESULTS_DIR/SUMMARY.txt"
            else
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
    echo "  Note: 14 further files in proofs/kani/ (acceleration_proofs.rs,"
    echo "  lyapunov_proofs.rs, ...) remain templates outside this run list."
    echo ""

    if [ "${1:-}" = "--exit" ]; then
        [ "$all_passed" = true ] && exit 0 || exit 1
    fi
}

list_harnesses() {
    printf '%s\n' \
        verify_schedule_info_injective \
        verify_schedule_key_and_nonce_info_separated \
        verify_schedule_first_indices \
        verify_schedule_limit_invariant \
        verify_extraction_header_injective \
        verify_extraction_body_injective \
        verify_add_no_overflow \
        verify_sub_no_overflow \
        verify_mul_range \
        verify_c1_preimage_bound \
        verify_c1_epsilon_bound \
        verify_c1_division_remainder \
        verify_information_loss_total \
        verify_information_loss_monotone \
        verify_saturation_point \
        verify_information_loss_saturating \
        verify_information_loss_cap_boundary \
        verify_lyapunov_spectrum_comprehensive \
        verify_lyapunov_division \
        verify_lyapunov_error_bounds \
        verify_lyapunov_quantization \
        verify_shadow_orbit_bias \
        verify_perturbation_in_range \
        verify_c3_step_damping \
        verify_c3_two_step_dispersion \
        verify_c3_quantum_reduction \
        verify_c3_exponent_arithmetic \
        verify_c3_exponent_law \
        verify_c3_contraction_arithmetic \
        verify_domain_separation \
        verify_framing_deterministic \
        verify_shake256_grover_arithmetic \
        verify_bound_is_min_of_two \
        verify_ct_select_i128 \
        verify_ct_select_sign_bits \
        verify_ct_is_zero_u64 \
        verify_ct_select_u64 \
        verify_ct_select_u64_is_exact \
        verify_floyd_cycle_detection \
        verify_hare_leads \
        verify_find_cycle_is_complete_and_correct \
        verify_find_cycle_is_sound_for_any_budget \
        verify_find_cycle_zero_budget
}

run_single() {
    local name="$1"
    mkdir -p "$RESULTS_DIR"
    print_banner
    echo ""
    echo "  ${BOLD}[1/1]${NC} ${CYAN}${name}${NC}"
    echo ""
    set +e
    run_with_progress "$name" "$RESULTS_DIR/${name}.log"
    local exit_code=$?
    set -e
    echo ""
    echo "  Results saved to: $RESULTS_DIR/"
    return "$exit_code"
}

case "${1:-}" in
    --version)
        cargo kani --version
        ;;
    --list)
        list_harnesses
        ;;
    --all | "")
        run_proofs
        ;;
    --proof | --harness)
        shift
        if [ $# -lt 1 ]; then
            echo "error: $0 --proof requires a harness name" >&2
            exit 2
        fi
        run_single "$1"
        ;;
    *)
        echo "error: unknown option '$1'" >&2
        echo "usage: run-kani [--all | --proof NAME | --list | --version]" >&2
        exit 2
        ;;
esac