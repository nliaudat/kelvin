#!/usr/bin/env bash
# Build and smoke-test the formal/analysis Docker images (Linux/macOS/CI).
#
#   ./scripts/formal-tools.sh --build
#   ./scripts/formal-tools.sh --smoke
#   ./scripts/formal-tools.sh --build --smoke proverif tamarin
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"
compose="$repo/docker/docker-compose.formal.yml"

all_tools=(proverif tamarin cryptoverif easycrypt smt rust-tools)
build=0
smoke=0
selected=()

for arg in "$@"; do
    case "$arg" in
        --build) build=1 ;;
        --smoke) smoke=1 ;;
        -h|--help)
            sed -n '2,8p' "$0"
            exit 0
            ;;
        *)
            if [[ " ${all_tools[*]} " == *" $arg "* ]]; then
                selected+=("$arg")
            else
                echo "unknown tool: $arg (valid: ${all_tools[*]})" >&2
                exit 2
            fi
            ;;
    esac
done

if [[ ${#selected[@]} -eq 0 ]]; then
    selected=("${all_tools[@]}")
fi

if [[ $build -eq 1 ]]; then
    for tool in "${selected[@]}"; do
        echo "== building $tool"
        docker compose -f "$compose" build "$tool"
    done
fi

if [[ $smoke -eq 1 ]]; then
    failed=0

    run_check() {
        local label="$1"
        shift
        echo "== $label"
        if docker compose -f "$compose" "$@"; then
            echo "   PASS"
        else
            echo "   FAIL" >&2
            failed=$((failed + 1))
        fi
    }

    # Like run_check, but also requires the tool's output to contain a pattern.
    # ProVerif and Tamarin exit 0 even on a falsified query/lemma.
    run_check_expect() {
        local label="$1" pattern="$2"
        shift 2
        echo "== $label"
        local out
        if out=$(docker compose -f "$compose" "$@" 2>&1) && grep -qE "$pattern" <<<"$out"; then
            echo "   PASS"
        else
            echo "   FAIL" >&2
            tail -n 15 <<<"$out" >&2
            failed=$((failed + 1))
        fi
    }

    for tool in "${selected[@]}"; do
        case "$tool" in
            proverif)
                run_check_expect "proverif smoke" 'RESULT not attacker\(key\[\]\) is true' \
                    run --rm -T proverif formal/proverif/kelvin_smoke.pv
                ;;
            tamarin)
                run_check_expect "tamarin smoke" 'key_secret \(all-traces\): verified' \
                    run --rm -T tamarin --prove formal/tamarin/kelvin_smoke.spthy
                ;;
            easycrypt)
                run_check "easycrypt smoke" run --rm -T easycrypt formal/easycrypt/smoke.ec
                run_check "easycrypt why3config" run --rm -T --entrypoint bash easycrypt -c 'easycrypt why3config'
                ;;
            cryptoverif)
                run_check "cryptoverif help" run --rm -T --entrypoint cryptoverif cryptoverif -help
                ;;
            smt)
                run_check "z3" run --rm -T --entrypoint z3 smt --version
                run_check "cvc5" run --rm -T --entrypoint cvc5 smt --version
                ;;
            rust-tools)
                run_check "cargo-fuzz" run --rm -T --entrypoint cargo rust-tools fuzz --version
                run_check "cargo-audit" run --rm -T --entrypoint cargo rust-tools audit --version
                run_check "cargo-deny" run --rm -T --entrypoint cargo rust-tools deny --version
                run_check "cargo-mutants" run --rm -T --entrypoint cargo rust-tools mutants --version
                run_check "miri" run --rm -T --entrypoint cargo rust-tools miri --version
                ;;
        esac
    done

    if [[ $failed -gt 0 ]]; then
        echo "$failed smoke check(s) failed" >&2
        exit 1
    fi
    echo "all selected smoke checks passed"
fi
