# Portability Log

Append-only log of anything in this project that ties it to the current host. Every entry here is a future migration task. The agent updates this file whenever it:

- Installs a system package (apt, brew, etc.)
- Hardcodes a path that won't exist on another host
- Configures a daemon at the system level
- Depends on a host-specific resource (specific architecture, kernel version)
- Uses a feature that's Linux-only or macOS-only or x86-only
- Configures the network at a level above the project tree

If the project is fully portable, this file should be near-empty.

**Format for each entry:**

```
## YYYY-MM-DD HH:MM UTC — [category]

**What:** the host-specific element
**Why needed:** justification
**Migration cost:** what to do on the new host
**Alternative:** is there a portable version we could use instead?
```

Categories: `system-package`, `hardcoded-path`, `system-daemon`, `architecture-specific`, `network-config`, `kernel-feature`.

---

## 2026-XX-XX XX:XX UTC — bootstrap

**What:** nftables egress allowlist installed at system level (`/etc/nftables.d/loom-lens-egress.nft`).
**Why needed:** Egress allowlist must be enforced at the kernel level, not by the agent itself.
**Migration cost:** Re-apply `nftables/egress.nft` on new host with `nft -f`.
**Alternative:** A userspace egress proxy (tinyproxy/squid) is portable but bypassable.

## 2026-XX-XX XX:XX UTC — bootstrap

**What:** systemd unit at `/etc/systemd/system/loom-lens-agent.service`.
**Why needed:** Resource limits on agent process; restart behavior.
**Migration cost:** Re-install unit on new host. On macOS, translate to launchd plist.
**Alternative:** Run the agent under tmux manually; lose automatic restart and resource limits.

---

## 2026-05-08 — system-daemon

**What:** systemd unit at `/etc/systemd/system/loom-lens-egress.service` (one-shot loader for the egress allowlist; tracks `RemainAfterExit=yes`).
**Why needed:** Without this, the egress rules are lost on reboot — `nftables.service` is inactive on this Rocky 9 box and there is no `/etc/nftables.conf`. Loading from a dedicated unit keeps the egress lifecycle independent of firewalld and of any future host-level nftables tooling.
**Migration cost:** Re-install on new host: `sudo cp systemd/loom-lens-egress.service /etc/systemd/system/ && sudo systemctl daemon-reload && sudo systemctl enable loom-lens-egress.service`. The unit's `ExecStartPost` resolves the refresh script from either `/opt/loom-lens/scripts/refresh-egress.sh` (production VPS path) or `/home/cc/ProjectAlpha/scripts/refresh-egress.sh` (this dev workstation), so the unit is portable across both layouts.
**Alternative:** Enable the system's `nftables.service` and add an include in `/etc/nftables.conf`. Slightly less explicit; the dedicated unit makes the dependency on `refresh-egress.sh` (which populates the IP set) visible.

## 2026-05-08 — network-config

**What:** Edited `/etc/resolv.conf` to put nftables-allowlist-permitted DNS resolvers first (1.1.1.1, 1.0.0.1, 9.9.9.9). Removed the cloud-init defaults (153.92.2.6, 8.8.8.8, 8.8.4.4) since they are not on the allowlist and would otherwise cause every lookup to time out before falling through to a working resolver. Backup at `/etc/resolv.conf.preloomlens-2026-05-08`.
**Why needed:** The egress allowlist intentionally restricts UDP 53 to a curated set of resolvers; resolv.conf has to point only at those.
**Migration cost:** On any new host: same edit. On a fresh production VPS the resolv.conf is empty enough that this is one paste. **Cloud-init may regenerate `/etc/resolv.conf` on reboot on this DigitalOcean instance** — if that happens, re-edit, or set `manage_resolv_conf: false` in `/etc/cloud/cloud.cfg.d/99-loom-lens.cfg` to make it persistent.
**Alternative:** Use `systemd-resolved` with a fixed upstream and let it own resolv.conf. Equivalent end state but adds a second daemon to reason about.

## 2026-05-08 — system-daemon

**What:** `/etc/cron.d/loom-lens-egress` (hourly invocation of `refresh-egress.sh` under root).
**Why needed:** nftables sets are populated from hostname → IP resolution, and CDN IPs rotate; without periodic refresh the allowlist drifts and "allowed" hosts start failing.
**Migration cost:** Re-install on new host with the path in the cron line pointing at the project root on that host.
**Alternative:** A systemd timer instead of cron. Equivalent.

## 2026-05-08 — system-daemon

**What:** `/etc/sudoers.d/loom-lens-egress` (NOPASSWD entry letting the `cc` user invoke `refresh-egress.sh`).
**Why needed:** Allows the agent's user to trigger an out-of-band refresh (e.g., after adding a hostname to the project's allowlist) without being prompted for a password.
**Migration cost:** Re-install on new host, substituting the agent user (`loom` on the production VPS, `cc` here) and the project root path.
**Alternative:** Skip — only refresh hourly via cron. Loses the ability to ad-hoc refresh after the agent edits the allowlist.

## 2026-05-09 — network-config

**What:** `/etc/gai.conf` with `precedence ::ffff:0:0/96 100`, making IPv4 preferred over IPv6 in glibc's resolver order.
**Why needed:** This Hostinger VM has uneven IPv6 path quality. Two failure modes observed: (a) CloudFront-fronted hosts (`sh.rustup.rs`, `mise.jdx.dev`) hand out v6 addresses across many `2600:9000:XXXX::/48` prefixes; some times out silently (we drop with no RST) before mise's 30s per-fetch budget falls through to v4. (b) The kernel drop log shows packets to `2605:72c0::/32` destinations during v6 connect attempts to hosts that resolve to entirely different addresses — looks like asymmetric routing or some Hostinger-side rewrite. Either way, native v6 cannot be relied on for outbound on this host.
**Migration cost:** Re-install on new host *only if needed*. The production VPS will likely have clean dual-stack and shouldn't need this. Reversible by `sudo rm /etc/gai.conf` (system falls back to glibc default precedence).
**Alternative:** Allowlist all v6 ranges any tool's CDN might use (CloudFront, Cloudflare, Fastly, etc. — open-ended) and accept that mise install will sometimes time out anyway because of (b). The gai.conf change sidesteps both issues with one rule. Trade-off: native-v6-only services (rare today) become unreachable.

---

(Subsequent entries appended below)
