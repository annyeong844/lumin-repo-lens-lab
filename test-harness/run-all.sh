#!/usr/bin/env bash
# run-all.sh — run every prompt in prompts/ (or a subset) and summarize.
#
# Usage:
#   ./run-all.sh                      # all prompts
#   ./run-all.sh prompts/positive     # positive only
#   ./run-all.sh prompts/negative     # negative only
#
# Exit codes:
#   0 — every test passed
#   1 — at least one test failed
#   2 — runtime error

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TARGET_DIR="${1:-$SCRIPT_DIR/prompts}"

# Resolve relative paths against the harness root
if [[ "$TARGET_DIR" != /* ]]; then
    TARGET_DIR="$SCRIPT_DIR/${TARGET_DIR#./}"
    TARGET_DIR="${TARGET_DIR%/}"
fi

if [ ! -d "$TARGET_DIR" ]; then
    echo "ERROR: target directory not found: $TARGET_DIR"
    exit 2
fi

# Always lint first — fast, free, catches structural mistakes before burning tokens
echo "=== Pre-flight: linting prompts/expectations ==="
node "$SCRIPT_DIR/lib/lint-prompts.mjs" || {
    echo "Lint failed. Fix prompt/expectations sync before running live tests."
    exit 2
}
echo ""

PASSED=0
FAILED=0
RESULTS=()

# Find all .txt prompts under target dir
PROMPTS=()
while IFS= read -r -d '' p; do
    PROMPTS+=("$p")
done < <(find "$TARGET_DIR" -name '*.txt' -print0 | sort -z)

if [ ${#PROMPTS[@]} -eq 0 ]; then
    echo "No prompt files under $TARGET_DIR"
    exit 2
fi

echo "=== Running ${#PROMPTS[@]} triggering test(s) ==="
echo ""

for prompt_full in "${PROMPTS[@]}"; do
    # Make path relative to harness root for verifier lookup
    prompt_rel="${prompt_full#$SCRIPT_DIR/}"

    echo "▶ $prompt_rel"

    if "$SCRIPT_DIR/run-test.sh" "$prompt_rel" 2>&1 | tail -20; then
        PASSED=$((PASSED + 1))
        RESULTS+=("✅ $prompt_rel")
    else
        FAILED=$((FAILED + 1))
        RESULTS+=("❌ $prompt_rel")
    fi

    echo ""
    echo "---"
    echo ""
done

echo ""
echo "=== Summary ==="
for r in "${RESULTS[@]}"; do
    echo "  $r"
done
echo ""
echo "Passed: $PASSED / $((PASSED + FAILED))"

if [ $FAILED -gt 0 ]; then
    exit 1
fi
exit 0
