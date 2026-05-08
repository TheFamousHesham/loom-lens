#!/usr/bin/env bash
# Snapshot the project state to a backup target.
# Run via cron (daily) or manually before risky operations.
#
# Tarballs the project tree (excluding regeneratable caches), pushes to
# the backup target via rsync over SSH. Backup target should be a separate
# host with its own credential boundary (BBL1 or another VPS).

set -euo pipefail

LOOM_LENS_ROOT="${LOOM_LENS_ROOT:-/opt/loom-lens}"
SNAPSHOT_DIR="$LOOM_LENS_ROOT/data/snapshots"
TIMESTAMP="$(date -u +%Y%m%d-%H%M%S)"
SNAPSHOT_NAME="loom-lens-snapshot-${TIMESTAMP}.tar.gz"
SNAPSHOT_PATH="$SNAPSHOT_DIR/$SNAPSHOT_NAME"

# Load .env for backup target settings.
if [[ -f "$LOOM_LENS_ROOT/.env" ]]; then
    set -a
    # shellcheck disable=SC1091
    source "$LOOM_LENS_ROOT/.env"
    set +a
fi

mkdir -p "$SNAPSHOT_DIR"

echo "Creating snapshot: $SNAPSHOT_NAME"

# Tar excludes — anything regeneratable.
tar -czf "$SNAPSHOT_PATH" \
    --exclude='./data/cache' \
    --exclude='./data/snapshots' \
    --exclude='./target' \
    --exclude='./node_modules' \
    --exclude='./.pnpm-store' \
    --exclude='./__pycache__' \
    --exclude='./.uv-cache' \
    --exclude='./dist-newstyle' \
    --exclude='./.stack-work' \
    --exclude='./.cargo' \
    --exclude='./.rustup' \
    --exclude='./.ghcup' \
    --exclude='./.mise' \
    --exclude='./.git/objects/pack/*.pack' \
    -C "$LOOM_LENS_ROOT" .

SIZE="$(du -h "$SNAPSHOT_PATH" | cut -f1)"
echo "Snapshot created: $SNAPSHOT_PATH ($SIZE)"

# Push to backup target if configured.
if [[ -n "${BACKUP_TARGET_HOST:-}" && -n "${BACKUP_TARGET_PATH:-}" ]]; then
    echo "Pushing to $BACKUP_TARGET_HOST:$BACKUP_TARGET_PATH/"
    SSH_KEY_OPT=""
    if [[ -n "${BACKUP_SSH_KEY:-}" && -f "${BACKUP_SSH_KEY}" ]]; then
        SSH_KEY_OPT="-e \"ssh -i $BACKUP_SSH_KEY\""
    fi
    rsync -avz --partial --progress \
        ${SSH_KEY_OPT} \
        "$SNAPSHOT_PATH" \
        "${BACKUP_TARGET_HOST}:${BACKUP_TARGET_PATH}/"
    echo "Push complete."
else
    echo "BACKUP_TARGET_HOST not set; snapshot stored locally only."
fi

# Rotate local snapshots: keep last 7.
echo "Rotating local snapshots (keeping last 7)..."
ls -1t "$SNAPSHOT_DIR"/loom-lens-snapshot-*.tar.gz 2>/dev/null | tail -n +8 | xargs -r rm -v

echo "Done."
