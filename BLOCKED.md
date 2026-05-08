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
- **nftables egress allowlist** — IN PROGRESS. User chose to apply with extensions (Rocky/EPEL mirrors). Updated `nftables/egress.nft` and `scripts/refresh-egress.sh` to add `mirrors.rockylinux.org`, `download.rockylinux.org`, `dl.fedoraproject.org`, `mirrors.fedoraproject.org` so `dnf` keeps working on this box. **Not yet applied at the kernel level.** Final go-ahead pending (separate AskUserQuestion below) since the apply step is sudo+disruptive: anything not on the allowlist (general web browsing, third-party Slack/Discord/Gmail clients, etc.) will start failing the moment the rules load.

## Still open after this round

- **Anthropic spend cap of $10.** User to set in console; agent cannot.
- **API key rotation** (recommended, not required). User can revoke and reissue at any point; update `.env`, no code change needed.
- **Production VPS provisioning** (`documentation/docs/PROVISIONING.md`). The dev workstation work above does not replace the production hardening; M3/M4 deploy still needs a dedicated VPS.
