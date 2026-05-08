#!/usr/bin/env bash
# Refresh the nftables egress allowlist from a list of allowed hostnames.
# Run hourly via cron. nftables can't resolve hostnames natively, so we
# maintain the IP sets out-of-band.
#
# Run as root via sudoers entry:
#   loom ALL=(root) NOPASSWD: /opt/loom-lens/scripts/refresh-egress.sh
#
# But the agent should NEVER call this directly. It runs as a cron job
# under root. If hostnames need to be added, the user edits this script.

set -euo pipefail

ALLOWED_HOSTS=(
    # Package registries
    "crates.io"
    "static.crates.io"
    "index.crates.io"
    "registry.npmjs.org"
    "registry.yarnpkg.com"
    "pypi.org"
    "files.pythonhosted.org"
    "proxy.golang.org"
    "sum.golang.org"
    "hackage.haskell.org"
    "downloads.haskell.org"

    # Code hosting and APIs
    "github.com"
    "api.github.com"
    "codeload.github.com"
    "objects.githubusercontent.com"
    "raw.githubusercontent.com"
    "api.anthropic.com"

    # Container registries
    "hub.docker.com"
    "registry-1.docker.io"
    "auth.docker.io"
    "production.cloudflare.docker.com"
    "ghcr.io"

    # System packages (Ubuntu/Debian)
    "archive.ubuntu.com"
    "security.ubuntu.com"
    "deb.debian.org"
    "esm.ubuntu.com"

    # System packages (Rocky / RHEL-likes / EPEL)
    "mirrors.rockylinux.org"
    "download.rockylinux.org"
    "dl.fedoraproject.org"
    "mirrors.fedoraproject.org"

    # Add backup target if set in environment.
)

if [[ -n "${BACKUP_TARGET_HOST:-}" ]]; then
    ALLOWED_HOSTS+=("$BACKUP_TARGET_HOST")
fi

V4_IPS=()
V6_IPS=()

for host in "${ALLOWED_HOSTS[@]}"; do
    while IFS= read -r ip; do
        V4_IPS+=("$ip")
    done < <(getent ahostsv4 "$host" 2>/dev/null | awk '{print $1}' | sort -u)

    while IFS= read -r ip; do
        V6_IPS+=("$ip")
    done < <(getent ahostsv6 "$host" 2>/dev/null | awk '{print $1}' | sort -u)
done

# Deduplicate.
mapfile -t V4_IPS < <(printf '%s\n' "${V4_IPS[@]}" | sort -u)
mapfile -t V6_IPS < <(printf '%s\n' "${V6_IPS[@]}" | sort -u)

# Build nft commands.
{
    echo "flush set inet loom_filter loom_allowed_ips_v4"
    if [[ ${#V4_IPS[@]} -gt 0 ]]; then
        echo "add element inet loom_filter loom_allowed_ips_v4 { $(IFS=,; echo "${V4_IPS[*]}") }"
    fi
    echo "flush set inet loom_filter loom_allowed_ips_v6"
    if [[ ${#V6_IPS[@]} -gt 0 ]]; then
        echo "add element inet loom_filter loom_allowed_ips_v6 { $(IFS=,; echo "${V6_IPS[*]}") }"
    fi
} | nft -f -

logger -t loom-egress "Refreshed: ${#V4_IPS[@]} v4 IPs, ${#V6_IPS[@]} v6 IPs"
