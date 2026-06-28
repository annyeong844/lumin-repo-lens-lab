#!/usr/bin/env bash
# run-test.sh — run claude -p with a naive prompt, then verify expectations.
#
# Usage:
#   ./run-test.sh <prompt-relative-path>
#   ./run-test.sh prompts/positive/audit-ko-dead-export.txt
#
# Environment:
#   PLUGIN_DIR      Path to the plugin root (defaults to harness parent dir).
#   MAX_TURNS       Max turns for claude -p (default: 10 — pre-write needs more
#                   turns than superpowers' 3 because the model has to extract
#                   intent, run the script, and synthesize advisory).
#   CLAUDE_BIN      Override claude CLI path (default: `claude`).
#   OUTPUT_BASE     Where to write logs (default: /tmp/auditing-skill-tests).
#
# Exit codes:
#   0 — verification passed
#   1 — verification failed (expectations not met)
#   2 — runtime error (claude CLI missing, etc.)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
HARNESS_ROOT="$SCRIPT_DIR"
PLUGIN_DIR="${PLUGIN_DIR:-$(cd "$HARNESS_ROOT/.." && pwd)}"
MAX_TURNS="${MAX_TURNS:-10}"
CLAUDE_BIN="${CLAUDE_BIN:-claude}"
OUTPUT_BASE="${OUTPUT_BASE:-/tmp/auditing-skill-tests}"

PROMPT_REL="$1"
if [ -z "$PROMPT_REL" ]; then
    echo "Usage: $0 <prompt-relative-path>"
    echo "Example: $0 prompts/positive/audit-ko-dead-export.txt"
    exit 2
fi

PROMPT_FILE="$HARNESS_ROOT/$PROMPT_REL"
if [ ! -f "$PROMPT_FILE" ]; then
    echo "ERROR: prompt file not found: $PROMPT_FILE"
    exit 2
fi
EXPECTATION_REL="${PROMPT_REL#prompts/}"

# Sanity: claude CLI present?
if ! command -v "$CLAUDE_BIN" >/dev/null 2>&1; then
    echo "ERROR: '$CLAUDE_BIN' not found on PATH."
    echo "       Install Claude Code or set CLAUDE_BIN=/path/to/claude."
    echo "       For offline schema validation, run: node lib/lint-prompts.mjs"
    exit 2
fi

TIMESTAMP=$(date +%s)
SAFE_NAME="${PROMPT_REL//\//_}"
SAFE_NAME="${SAFE_NAME%.txt}"
OUTPUT_DIR="$OUTPUT_BASE/$TIMESTAMP/$SAFE_NAME"
mkdir -p "$OUTPUT_DIR"
LOG_FILE="$OUTPUT_DIR/stream.json"

echo "=== Skill Triggering Test ==="
echo "Prompt:     $PROMPT_REL"
echo "Plugin dir: $PLUGIN_DIR"
echo "Max turns:  $MAX_TURNS"
echo "Output:     $OUTPUT_DIR"
echo ""

# Copy prompt for forensic reference
cp "$PROMPT_FILE" "$OUTPUT_DIR/prompt.txt"

PROMPT=$(cat "$PROMPT_FILE")

# Run Claude. We do NOT exit on non-zero here — the model may legitimately exit
# nonzero on negative tests (no skill matched). Verifier decides pass/fail.
echo "Running claude -p..."
CLAUDE_ARGS=(
    -p "$PROMPT"
    --plugin-dir "$PLUGIN_DIR"
    --dangerously-skip-permissions
    --max-turns "$MAX_TURNS"
    --verbose
    --output-format stream-json
)

if command -v timeout >/dev/null 2>&1 && timeout --version >/dev/null 2>&1; then
    timeout 300 "$CLAUDE_BIN" "${CLAUDE_ARGS[@]}" > "$LOG_FILE" 2>&1 || true
else
    "$CLAUDE_BIN" "${CLAUDE_ARGS[@]}" > "$LOG_FILE" 2>&1 || true
fi

echo ""
echo "=== Verification ==="

# Run verifier
node "$HARNESS_ROOT/lib/verify.mjs" "$LOG_FILE" "$EXPECTATION_REL"
VERIFY_STATUS=$?

echo ""
echo "Log: $LOG_FILE"

exit $VERIFY_STATUS
