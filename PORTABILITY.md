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

(Subsequent entries appended below)
