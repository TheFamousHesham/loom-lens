#!/usr/bin/env bash
# Migrate the project to a new host.
# Run on the SOURCE host. Pushes a snapshot to the target, then provides
# instructions for the target host.
#
# Usage:
#   ./scripts/migrate.sh user@new-host:/opt/loom-lens
#
# Pre-conditions on the target host:
#   - 'loom' user created with no sudo
#   - sshd hardened
#   - mise installed
#   - docker installed
#   - nftables active with the egress allowlist applied
#   - This SSH key has access to /opt/loom-lens on the target
#   - /opt/loom-lens on target is empty or non-existent

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "Usage: $0 user@host:/path/to/loom"
    echo ""
    echo "Example: $0 loom@bbl1.tailnet:/opt/loom-lens"
    exit 1
fi

TARGET="$1"
LOOM_LENS_ROOT="${LOOM_LENS_ROOT:-/opt/loom-lens}"

echo "=== Loom Migration ==="
echo "Source: $LOOM_LENS_ROOT (this host)"
echo "Target: $TARGET"
echo ""

# Pre-flight checks.
echo "[1/5] Pre-flight checks..."

# Check git is clean.
cd "$LOOM_LENS_ROOT"
if [[ -n "$(git status --porcelain)" ]]; then
    echo "ERROR: git working tree is dirty. Commit or stash first."
    exit 1
fi

# Check git is pushed.
if [[ -n "$(git log @{u}.. 2>/dev/null)" ]]; then
    echo "ERROR: unpushed commits on current branch. Push first."
    exit 1
fi

echo "Git clean and pushed: OK"

# Check no sensitive files are about to be transferred.
echo "[2/5] Auditing for sensitive files..."
SENSITIVE_FOUND=0
while IFS= read -r f; do
    echo "  ! $f"
    SENSITIVE_FOUND=1
done < <(find "$LOOM_LENS_ROOT" -type f \( \
    -name '*.key' -o \
    -name '*.pem' -o \
    -name 'credentials*' -o \
    -name '.env' \
    \) -not -path '*/node_modules/*' -not -path '*/target/*')

if [[ $SENSITIVE_FOUND -eq 1 ]]; then
    echo ""
    echo "Sensitive files found in project tree. .env will NOT be transferred"
    echo "automatically. You must copy it separately to the target host"
    echo "via a different channel (out-of-band)."
    echo ""
    read -p "Continue with migration (excluding .env)? [y/N] " -n 1 -r
    echo
    [[ ! $REPLY =~ ^[Yy]$ ]] && exit 1
fi

# Create the snapshot.
echo "[3/5] Creating migration snapshot..."
"$LOOM_LENS_ROOT/scripts/snapshot.sh"
LATEST_SNAPSHOT="$(ls -1t "$LOOM_LENS_ROOT/data/snapshots"/loom-lens-snapshot-*.tar.gz | head -1)"
echo "Snapshot: $LATEST_SNAPSHOT"

# Transfer.
echo "[4/5] Transferring to $TARGET..."
TARGET_HOST="${TARGET%:*}"
TARGET_PATH="${TARGET#*:}"

ssh "$TARGET_HOST" "mkdir -p $TARGET_PATH"

rsync -avz --partial --progress \
    "$LATEST_SNAPSHOT" \
    "${TARGET_HOST}:${TARGET_PATH}/"

# Print instructions for the target host.
echo "[5/5] Source-side migration complete."
echo ""
echo "=== NEXT STEPS — RUN ON TARGET HOST ==="
echo ""
echo "ssh $TARGET_HOST"
echo "cd $TARGET_PATH"
echo "tar xzf $(basename "$LATEST_SNAPSHOT")"
echo "rm $(basename "$LATEST_SNAPSHOT")"
echo ""
echo "# Copy .env from secure location (NOT the snapshot):"
echo "scp ./.env $TARGET_HOST:$TARGET_PATH/.env"
echo "ssh $TARGET_HOST chmod 600 $TARGET_PATH/.env"
echo ""
echo "# Run target-side bootstrap:"
echo "ssh $TARGET_HOST"
echo "cd $TARGET_PATH"
echo "./scripts/init.sh --resume"
echo ""
echo "# Then start the agent on the new host:"
echo "tmux new-session -s loom"
echo "./scripts/run-agent.sh"
echo ""
echo "=== POST-MIGRATION CHECKLIST ==="
echo "[ ] Old VPS still running until target verified"
echo "[ ] STATUS.md visible on target"
echo "[ ] mise tools resolve on target"
echo "[ ] docker compose up -d tools succeeds on target"
echo "[ ] git push from target works"
echo "[ ] Anthropic API key works from target"
echo "[ ] Agent reaches the next checkpoint successfully"
echo "[ ] Then: rotate Anthropic API key (old one was on the source)"
echo "[ ] Then: destroy source VPS"
