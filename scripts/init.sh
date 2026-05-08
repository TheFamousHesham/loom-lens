#!/usr/bin/env bash
# Bootstrap a fresh host for the Loom Lens project.
# Run as the loom user (NOT root) after host has been provisioned per documentation/docs/PROVISIONING.md.

set -euo pipefail

# Resolve project root from script location so this works in /opt/loom-lens (production)
# and /home/cc/ProjectAlpha (dev) without requiring an explicit env export.
LOOM_LENS_ROOT="${LOOM_LENS_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
STATE_FILE="$LOOM_LENS_ROOT/.init-state"
RESUME=false

for arg in "$@"; do
    case "$arg" in
        --resume) RESUME=true ;;
        *) echo "Unknown arg: $arg"; exit 1 ;;
    esac
done

cd "$LOOM_LENS_ROOT"
touch "$STATE_FILE"

step() {
    local name="$1"
    if $RESUME && grep -qx "$name" "$STATE_FILE"; then
        echo "[skip] $name (already done)"
        return 1
    fi
    echo "[run]  $name"
    return 0
}

mark_done() { echo "$1" >> "$STATE_FILE"; }

# ============================================================================
# 1. Sanity check.
# ============================================================================

if step "sanity-check"; then
    if [[ "$(whoami)" == "root" ]]; then
        echo "ERROR: do not run init.sh as root."
        exit 1
    fi
    if [[ ! -f "$LOOM_LENS_ROOT/.env" ]]; then
        echo "ERROR: .env file missing. Copy .env.example to .env and fill in."
        exit 1
    fi
    if ! command -v mise &>/dev/null; then
        echo "ERROR: mise not installed."
        exit 1
    fi
    # Verify egress allowlist by trying a non-allowed host.
    if curl --connect-timeout 3 -s -o /dev/null https://example.com 2>/dev/null; then
        echo "WARNING: egress to example.com succeeded — nftables allowlist may not be active."
    fi
    mark_done "sanity-check"
fi

# ============================================================================
# 2. Install pinned toolchains.
# ============================================================================

if step "mise-install"; then
    cd "$LOOM_LENS_ROOT"
    mise install
    mise list
    mark_done "mise-install"
fi

# ============================================================================
# 3. Configure git identity from .env.
# ============================================================================

if step "git-config"; then
    cd "$LOOM_LENS_ROOT"
    set -a
    # shellcheck disable=SC1091
    source .env
    set +a

    git config user.name "${GIT_AUTHOR_NAME}"
    git config user.email "${GIT_AUTHOR_EMAIL}"

    # Conventional commits hook (rejects malformed messages).
    cat > .git/hooks/commit-msg <<'EOF'
#!/bin/sh
msg_file=$1
first_line=$(head -1 "$msg_file")
if ! echo "$first_line" | grep -qE '^(feat|fix|refactor|test|docs|chore|security|perf|build|ci)(\(.+\))?: .+'; then
    echo "ERROR: commit message must follow conventional commits format."
    echo "       Examples: 'feat: add Python parser', 'fix(mcp): handle empty repos'"
    exit 1
fi
EOF
    chmod +x .git/hooks/commit-msg

    # Pre-commit hook: block commits containing high-entropy strings that look like secrets.
    cat > .git/hooks/pre-commit <<'EOF'
#!/bin/sh
# Block commits containing potential secrets.
diff=$(git diff --cached)
if echo "$diff" | grep -iE '(api[_-]?key|secret|token|password|bearer)' | \
   grep -vE '(example|placeholder|template|TODO|FIXME|test|fixture|//.*key|#.*key|REQUIRED|OPTIONAL)' | \
   grep -E '=\s*"[A-Za-z0-9_+/=-]{20,}"|=\s*[A-Za-z0-9_+/=-]{20,}'; then
    echo "ERROR: commit appears to contain a credential."
    echo "       If this is a false positive, review carefully then bypass with --no-verify."
    echo "       (Better yet: don't bypass; sanitize the diff.)"
    exit 1
fi
EOF
    chmod +x .git/hooks/pre-commit

    mark_done "git-config"
fi

# ============================================================================
# 4. Verify GitHub access.
# ============================================================================

if step "github-access"; then
    set -a
    # shellcheck disable=SC1091
    source "$LOOM_LENS_ROOT/.env"
    set +a

    GITHUB_KEY="${GITHUB_DEPLOY_KEY_PATH:-/home/loom/.ssh/loom_lens_deploy}"
    if [[ ! -f "$GITHUB_KEY" ]]; then
        echo "ERROR: GitHub deploy key not found at $GITHUB_KEY"
        echo "       Generate: ssh-keygen -t ed25519 -f $GITHUB_KEY -N ''"
        exit 1
    fi
    chmod 600 "$GITHUB_KEY"

    mkdir -p "$HOME/.ssh"
    chmod 700 "$HOME/.ssh"
    if ! grep -q "Host github.com" "$HOME/.ssh/config" 2>/dev/null; then
        cat >> "$HOME/.ssh/config" <<EOF

Host github.com
    User git
    IdentityFile $GITHUB_KEY
    IdentitiesOnly yes
EOF
    fi

    ssh-keyscan -H github.com >> "$HOME/.ssh/known_hosts" 2>/dev/null

    if ssh -T git@github.com 2>&1 | grep -q "successfully authenticated"; then
        echo "GitHub SSH: OK"
    else
        echo "WARNING: GitHub SSH did not authenticate. Verify deploy key is added with write access."
    fi

    mark_done "github-access"
fi

# ============================================================================
# 5. Verify environment.
# ============================================================================

if step "verify"; then
    cd "$LOOM_LENS_ROOT"
    echo ""
    echo "=== Environment Check ==="
    echo "Working dir:    $(pwd)"
    echo "User:           $(whoami)"
    echo "Git remote:     $(git remote get-url origin 2>/dev/null || echo '(not configured)')"
    echo "Git author:     $(git config user.name) <$(git config user.email)>"
    echo "mise tools:"
    mise list | sed 's/^/  /'
    echo "Disk free:      $(df -h /opt | awk 'NR==2 {print $4 " on " $1}')"
    echo "Memory free:    $(free -h | awk '/^Mem:/ {print $4 " of " $2}')"
    echo "========================="
    mark_done "verify"
fi

echo ""
echo "Bootstrap complete."
echo ""
echo "Next:"
echo "  1. Open a tmux session: tmux new-session -s loom-lens"
echo "  2. Run the agent:       ./scripts/run-agent.sh"
