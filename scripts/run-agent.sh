#!/usr/bin/env bash
# Run the Claude Code agent in autonomous mode for Loom Lens.

set -euo pipefail

LOOM_LENS_ROOT="${LOOM_LENS_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
cd "$LOOM_LENS_ROOT"

if [[ -f .env ]]; then
    set -a
    # shellcheck disable=SC1091
    source .env
    set +a
fi

if [[ -z "${ANTHROPIC_API_KEY:-}" ]]; then
    echo "ERROR: ANTHROPIC_API_KEY not set in .env"
    exit 1
fi

if ! command -v cargo &>/dev/null; then
    echo "ERROR: cargo not in PATH. Is mise activated?"
    echo "       Try: eval \"\$(mise activate bash)\""
    exit 1
fi

for f in STATUS.md BLOCKED.md SECURITY.md PORTABILITY.md CLAUDE.md; do
    if [[ ! -f "$f" ]]; then
        echo "ERROR: $f missing. Run ./scripts/init.sh first."
        exit 1
    fi
done

LOG_DIR="$LOOM_LENS_ROOT/.logs"
mkdir -p "$LOG_DIR"
SESSION_LOG="$LOG_DIR/session-$(date -u +%Y%m%d-%H%M%S).log"

{
    echo "=== Agent session start: $(date -u) ==="
    echo "Host: $(hostname)"
    echo "User: $(whoami)"
    echo "Loom Lens root: $LOOM_LENS_ROOT"
    echo "Git HEAD: $(git rev-parse --short HEAD 2>/dev/null || echo 'no commits')"
    echo ""
} | tee -a "$SESSION_LOG"

# Pre-flight snapshot.
echo "[pre-flight] Taking snapshot..."
"$LOOM_LENS_ROOT/scripts/snapshot.sh" >> "$SESSION_LOG" 2>&1 || {
    echo "WARNING: snapshot failed. Continuing anyway."
}

claude \
    --dangerously-skip-permissions \
    --working-dir "$LOOM_LENS_ROOT" \
    --output-format text \
    --max-turns 0 \
    "Read CLAUDE.md fully. Then read STATUS.md to understand current state. Then continue work according to the checkpoint plan in CHECKPOINTS.md. Stop at the next checkpoint." \
    2>&1 | tee -a "$SESSION_LOG"

EXIT_CODE=${PIPESTATUS[0]}

{
    echo ""
    echo "=== Agent session end: $(date -u) ==="
    echo "Exit code: $EXIT_CODE"
} | tee -a "$SESSION_LOG"

if grep -q "CHECKPOINT.*REACHED.*AWAITING USER" "$SESSION_LOG"; then
    echo ""
    echo "############################################"
    echo "#                                          #"
    echo "#   CHECKPOINT REACHED — REVIEW NEEDED     #"
    echo "#                                          #"
    echo "############################################"
    echo ""
    echo "Read STATUS.md, then respond in chat."
fi

exit "$EXIT_CODE"
