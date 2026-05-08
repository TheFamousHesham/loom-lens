# Status

Updated at end of every session and at every checkpoint by the agent. Read this at the start of every session before doing anything else.

---

## Last updated

**Timestamp:** Not yet started
**Session:** 0 (pre-work)
**Agent:** —
**Current commit:** initial skeleton

## Current phase

Pre-Checkpoint 1. Project skeleton committed. Awaiting kickoff.

## What's done

- Project structure created
- All design documents (`CLAUDE.md`, `README.md`, `ARCHITECTURE.md`, `CHECKPOINTS.md`)
- Open-source scaffolding (`LICENSE`, `CONTRIBUTING.md`)
- Toolchain pinned (`mise.toml`)
- Security infrastructure (`nftables/egress.nft`, `systemd/loom-lens-agent.service`)
- Bootstrap and operational scripts in `scripts/`
- ADR template and three pre-written ADRs covering tech stack, effect strategy, distribution
- CI workflow skeleton in `.github/workflows/ci.yml`
- State files initialized (this file, `BLOCKED.md`, `SECURITY.md`, `PORTABILITY.md`)

## What's in progress

Nothing. Awaiting agent kickoff on Checkpoint 1.

## What's next

When the agent begins:
1. Read `CLAUDE.md` in full.
2. Read `ARCHITECTURE.md` and `CHECKPOINTS.md`.
3. Confirm environment health (mise tools resolve, network egress allowlist active, .env populated, GitHub deploy key works).
4. Begin Checkpoint 1 deliverables: complete the four ADRs, write `docs/api.md` with full MCP tool schemas, draft the three per-language effect rules files, prepare test fixtures, sketch the viewer mockup.
5. Commit, push, print checkpoint banner, stop.

## Open decisions

(Pending Checkpoint 1 review by user)
- Final tech stack confirmation: Rust + React + Cytoscape (defaults proposed).
- Effect category set: 9 categories (`Pure`, `Net`, `IO`, `Mut`, `Throw`, `Async`, `Random`, `Time`, `Foreign`) — add or remove?
- MCP tool surface: 4 tools — too many, too few?
- Effect rule completeness: do the drafts match the user's intent on what the lens should detect?

## Token spend this session

—

## Notes

This is the bootstrap state. First real session begins on the user's go-ahead.
