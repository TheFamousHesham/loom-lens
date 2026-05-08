# 0005 — Repository Layout: Root-First, with `documentation/` for Living Design Docs

**Date:** 2026-05-08
**Status:** Accepted
**Reviewed at checkpoint:** 1

## Context

At kickoff, the project was delivered as a tarball alongside flat duplicates of six `.md` files in a single `documentation/` directory. The intended layout — implicit in the tarball, the systemd unit, `init.sh`, and the cross-references inside the design docs — was a normal repo with state files and infrastructure at the root and a `docs/` subdirectory for the design library. The flat-file presentation was an artifact of how the project was handed off, not the desired final shape.

Two layouts were credible:

- **Everything-at-root** (the tarball default). All `.md`s and infra at the repo root, with only API/ADR/effect-rules nested in a single `docs/` subdir.
- **Two-tier** (chosen). State/infra/public-surface files at the root; the longer-form design library — ARCHITECTURE.md, CHECKPOINTS.md, ADRs, API spec, effect rules, provisioning guide — under `documentation/`.

The constraint set:

- A first-time visitor (human or scanner) lands on the repo root. They expect to see README, LICENSE, CONTRIBUTING, install hint, and a manifest of files they may need to touch (`.env.example`, `mise.toml`, etc.). Burying README behind a directory is hostile.
- The agent rereads CLAUDE.md every session. CLAUDE.md belongs at the root so it's the first thing a fresh `ls` surfaces.
- State files that the agent appends to between checkpoints (STATUS.md, BLOCKED.md, SECURITY.md, PORTABILITY.md) are operationally hot. They're not "documentation" in the sense of design records; they are mutable logs the operator reads at a glance.
- Infrastructure (`scripts/`, `nftables/`, `systemd/`, `.github/`) is consumed by tooling that expects root-relative paths (the systemd unit's `WorkingDirectory=/opt/loom-lens`, GitHub's autodiscovery of `.github/workflows/`).
- The longer design documents — ADRs, the API contract, effect-rule documents, the provisioning runbook — are reference material. They're touched at design checkpoints, not on every session. Their length and inter-linking benefit from a dedicated subtree.

## Decision

Two-tier layout, with the boundary drawn by *change frequency × audience* rather than by content type:

```
/home/cc/ProjectAlpha/                  (project root; /opt/loom-lens on production VPS)
├── CLAUDE.md                           # agent operating instructions
├── README.md                           # public-facing entry point
├── LICENSE                             # MIT
├── CONTRIBUTING.md                     # external contributor surface
├── STATUS.md                           # mutable: agent updates each session
├── BLOCKED.md                          # mutable: append-only blocker log
├── SECURITY.md                         # mutable: append-only audit log
├── PORTABILITY.md                      # mutable: append-only host-tied log
├── .env.example                        # operational config template
├── .gitignore
├── mise.toml                           # toolchain pin
├── scripts/                            # bootstrap, run-agent, snapshot, migrate, refresh-egress
├── nftables/                           # egress allowlist
├── systemd/                            # production unit file
├── .github/                            # workflows (CI, release-on-dispatch)
├── crates/                             # Rust workspace (M1+)
├── frontend/                           # React/Vite (M1+)
├── tests/                              # fixtures + unit tests
├── .git/
└── documentation/
    ├── ARCHITECTURE.md                 # design contract; deviations require an ADR
    ├── CHECKPOINTS.md                  # five-checkpoint plan
    └── docs/
        ├── api.md                      # MCP tool contract
        ├── PROVISIONING.md             # production VPS runbook
        ├── decisions/                  # ADRs (this one is 0005)
        │   ├── 0000-template.md
        │   ├── 0001-tech-stack.md
        │   ├── 0002-effect-inference.md
        │   ├── 0003-distribution.md
        │   ├── 0004-mcp-tool-api.md
        │   └── 0005-repo-layout.md
        ├── effect-rules/               # per-language inference patterns
        │   ├── python.md
        │   ├── typescript.md
        │   └── rust.md
        └── viewer-mockup.md            # text-based wireframes for the three modes
```

Cross-document references in files that live at the root use the absolute-from-root form (`documentation/CHECKPOINTS.md`, `documentation/docs/decisions/...`). Cross-references between files inside `documentation/` may stay relative.

## Alternatives considered

- **Everything-at-root (the tarball default).** Simpler. But twelve `.md`s plus three subdirectories of design docs makes the root noisy on every `ls`, and the operationally hot files (STATUS, BLOCKED) drown in the design library. Rejected.
- **Everything-in-`docs/`, including state and CLAUDE.md.** Symmetric and tidy on the root. But it puts CLAUDE.md, the agent's bible, behind a directory that the kickoff script doesn't `cd` into; and it inverts the convention every GitHub visitor expects (README at root). Rejected.
- **Three-tier (`docs/` for spec, `state/` for mutable logs, root for everything else).** Clean separation but adds a directory the user does not need. The four state files at root are not enough mass to warrant a directory of their own. Rejected as over-engineered.
- **`design/` instead of `documentation/`.** Cosmetic. The kickoff already used `documentation/`; renaming creates a churn diff with no signal.

## Consequences

### Positive
- A first-time visitor sees README, LICENSE, CONTRIBUTING, and a small set of clearly-named state files. Anything they want to touch (`.env.example`, `mise.toml`, `scripts/`) is within reach.
- The agent sees CLAUDE.md, STATUS.md, BLOCKED.md on the same screen as `git status`. That is the loop it operates inside.
- The design library is dense but bounded. Following one cross-link inside `documentation/` doesn't require crossing a directory boundary.
- Infrastructure paths (`.github/workflows/`, `systemd/loom-lens-agent.service`, `scripts/init.sh`) keep their conventional locations. No tool needs to be reconfigured.

### Negative
- Two-tier requires absolute-from-root paths in cross-references from root files into `documentation/`. Slightly more verbose than `./CHECKPOINTS.md` would have been.
- New contributors must learn that ARCHITECTURE.md is *not* at root. Mitigated by a one-line pointer in README and by the fact that the design library is internally well-cross-linked.

### Risks
- Drift over time: as the project grows, files may end up in the wrong tier. Mitigated by this ADR — when in doubt, ask: *will an operator/visitor read this on a `ls`-frequency cadence (root) or only when reasoning about design (`documentation/`)?*
- Tooling that hardcodes `docs/` as a path (some doc generators, GitHub Pages defaults) won't find `documentation/docs/`. v0.1 has no such tooling; revisit if/when GitHub Pages is enabled.

## References

- This ADR was filed alongside the move itself; the diff in the same commit is the implementation.
- Predecessor: implicit layout in the tarball delivered at kickoff.
- See also: `documentation/ARCHITECTURE.md` §3 (crate layout) for the parallel decision on the Rust workspace.
