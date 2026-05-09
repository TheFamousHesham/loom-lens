# Blocked

Append-only log of things blocking the agent that need user input. The agent writes here and stops; the user reviews at the next checkpoint or proactively.

**Each entry includes:**

```
## YYYY-MM-DD HH:MM UTC — [short title]

**Status:** open | resolved
**Blocking:** what work is paused
**Issue:** what's blocking
**What I tried:** approaches attempted
**What I need:** the specific input or decision required
**Resolution:** (filled in when resolved)
```

Do not delete entries. Mark as `resolved` and append the resolution.

---

## 2026-05-08 — Environment is dev workstation, not the production VPS the kickoff envisions

**Status:** open
**Blocking:** Step 3 environment verification cannot pass production-style checks. Checkpoint 1 work (design only) proceeds regardless. Future checkpoints (M1+ which need to actually compile Rust) will be blocked until either (a) the user provisions per `documentation/docs/PROVISIONING.md` on the target VPS, or (b) the agent is permitted to install toolchains in `/home/cc/ProjectAlpha/`.

**Issue:** Verification subitems and their status:
- `mise list` — **fail**: `mise` is not installed on this host. Required by mise.toml for pinned Rust 1.85.0 / Node 22.11.0 / pnpm 9.15.0. Not strictly needed for Checkpoint 1 (design-only).
- `.env` exists with 600 perms and a valid `ANTHROPIC_API_KEY` — **fail**: only `.env.example` exists. Not needed for design-only Checkpoint 1 (no API calls). Required from M1.
- nftables egress allowlist active (`curl https://example.com` should fail) — **fail**: `curl https://example.com` returned HTTP 200; nftables is unprivileged here (`nft list ruleset` returns "Operation not permitted"). The kernel-level allowlist is part of the production VPS hardening per `documentation/docs/PROVISIONING.md`, not present in dev workstation.
- `git remote` set to `git@github.com:TheFamousHesham/loom-lens.git` — **fixed**: remote added; reachable via SSH (the user's personal `id_ed25519` key authenticates as `TheFamousHesham`, so all git ops will succeed once the GitHub repo exists).
- `git push` succeeds — **fail**: the GitHub repository at `TheFamousHesham/loom-lens` does not yet exist (404 from `api.github.com/repos/TheFamousHesham/loom-lens`). Cannot push.
- Pre-commit and commit-msg hooks installed — **fixed**: hooks copied verbatim from `scripts/init.sh` and smoke-tested (rejects non-conventional messages, accepts `feat: ...`).
- Anthropic API key hard spend cap — **unverifiable from this side**: cap setting is in the Anthropic console; user must confirm.

**What I tried:**
- Took ownership of the project tree once via sudo (under user authorization given in chat) so subsequent ops run as `cc:cc`. No further sudo used.
- Confirmed SSH auth as TheFamousHesham via `ssh -T git@github.com`.
- Probed `gh` CLI: authenticated as TheFamousHesham with full `repo`, `delete_repo`, `admin:public_key` scopes (i.e., the agent could create the public repo with `gh repo create` if instructed).
- Did not create the repo: creating a public GitHub repository is permanent and visible — saving that decision for the user.

**What I need:**
1. Decision on GitHub repo creation. Three reasonable paths:
   - (a) you create `TheFamousHesham/loom-lens` (public, MIT) on github.com manually, then I `git push -u origin main`;
   - (b) authorize me to run `gh repo create TheFamousHesham/loom-lens --public --description "See your codebase as a graph: structure, effects, and content-addressed identity." --license MIT --source=. --push`;
   - (c) defer the push until a later checkpoint and ship Checkpoint 1 locally only.
2. Confirmation that the Anthropic API key has a hard spend cap configured in the console.
3. For M1+: provision per `documentation/docs/PROVISIONING.md` on the target VPS, or tell me to install `mise` etc. into this dev workstation tree.

**Resolution (2026-05-08, in-session):**
- **GitHub repo creation** — RESOLVED. User authorized `gh repo create`. Repo created at https://github.com/TheFamousHesham/loom-lens (public, no LICENSE override since one was already in the source). The Checkpoint 1 commit is pushed; default branch is `main`.
- **mise install on this dev workstation** — RESOLVED. mise v2026.5.3 installed at `~/.local/share/mise/`, symlinked into `~/.local/bin/mise`, activated from `~/.bashrc`. The project's `mise.toml` is trusted; `mise list` shows the 8 pinned tools as "missing" until `mise install` runs (deferred — the actual toolchain download is ~700 MB plus cargo-tool compile time, kicked off by the user when convenient).
- **`.env` populated and locked** — RESOLVED. Created from `.env.example`, set to mode 600, gitignored. Placeholders filled in for this dev environment: project root, deploy key path (using the user's `~/.ssh/id_ed25519` rather than a separate per-repo deploy key, which is appropriate for dev), git author identity. ANTHROPIC_API_KEY filled in (see next item).
- **Anthropic auth** — RESOLVED for the key, OPEN for the spend cap. The user is on the Max 200 plan; for the headless agent loop they generated a project-scoped API key, which is now in `.env` and confirmed working against `/v1/models` (HTTP 200, latest model list returned). The key was sent via chat — recommend rotating it after the project is up to keep live credentials out of the local Claude Code transcript. **Spend cap of $10 is not yet set; that's a console-only action (Settings → Billing → Usage limits at console.anthropic.com).**
- **nftables egress allowlist** — RESOLVED. User picked "Apply anyway; reorder resolv.conf" after seeing that this box has firewalld + crowdsec + a DO-managed table. Concrete state:
  - Two pre-existing bugs in `nftables/egress.nft` were fixed during apply (NTP rule used unresolvable hostnames in a set literal; ICMPv6 type names used legacy long-form not accepted by nft 1.0.9).
  - Rules loaded into the kernel as table `inet loom_filter`. `loom_allowed_ips_v4` has 69 IPs; `loom_allowed_ips_v6` has 76. Verified: `api.anthropic.com` HTTP 200, `github.com` HTTP 200, `dl.fedoraproject.org` HTTP 302; `example.com` and `stackoverflow.com` time out at connect — the allowlist works as designed.
  - `/etc/resolv.conf` reordered to allowlist-permitted resolvers (1.1.1.1, 1.0.0.1, 9.9.9.9) only. Backup at `/etc/resolv.conf.preloomlens-2026-05-08`.
  - Persistence: `/etc/systemd/system/loom-lens-egress.service` installed and enabled (will load rules + run `refresh-egress.sh` at next boot). Hourly cron at `/etc/cron.d/loom-lens-egress` keeps the IP set fresh against CDN rotation. `/etc/sudoers.d/loom-lens-egress` grants the `cc` user NOPASSWD access to `refresh-egress.sh` for ad-hoc refresh.
  - Coexists with the host's existing firewalld, crowdsec, and DigitalOcean-managed tables — separate `loom_filter` table, no overlap.
  - Full audit trail in `SECURITY.md` (egress-update entry); host-tied changes in `PORTABILITY.md`; rollback recipe in both.

## Still open after this round

- ~~**Anthropic spend cap of $10.**~~ — RESOLVED by user choice (2026-05-09): autocharge is off and the account is funded with $10 only, so an explicit per-key spend cap isn't needed; the account-level balance is itself the cap.
- ~~**API key rotation.**~~ — RESOLVED by user choice (2026-05-09): treated as a development-only key with the same $10 ceiling; rotation deferred until the project graduates to a production posture.
- **Production VPS provisioning** (`documentation/docs/PROVISIONING.md`). The dev workstation work above does not replace the production hardening; M3/M4 deploy still needs a dedicated VPS.
- **Mise tool installation.** Egress allowlist initially blocked `mise-versions.jdx.dev`; allowlist extended on 2026-05-09 with `mise-versions.jdx.dev`, `mise.jdx.dev`, `static.rust-lang.org`, `nodejs.org`. `mise install` now runs; full install (Rust + Node + cargo tools) takes several minutes.
