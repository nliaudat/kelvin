#!/usr/bin/env bash
# Verify the Lean suite the way CI would.
#
# 1. Build every module (the kernel checks every proof during elaboration).
# 2. Fail if any proof escape hatch is present.
# 3. Print the theorem count.
#
# Usage: ./check.sh
set -euo pipefail
cd "$(dirname "$0")"

echo "== lake build =="
lake build

echo "== scanning for proof escape hatches =="
if grep -RnE '\b(sorry|admit|axiom|native_decide)\b' src --include='*.lean'; then
    echo "error: forbidden proof escape hatch found" >&2
    exit 1
fi
echo "OK: no sorry/admit/axiom/native_decide"

count=$(grep -RhE '^[[:space:]]*(theorem|lemma)[[:space:]]' src --include='*.lean' | wc -l)
echo "== ${count} theorems/lemmas checked =="
