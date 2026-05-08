# Checkpoints

Five checkpoints structure this project. Tighter cadence than Loom main given the smaller scope. The agent works autonomously between checkpoints and stops at each to await user review.

---

## Checkpoint 1 — Design lock

**No code yet.**

**Deliverables:**
- All ADRs filed in `docs/decisions/`:
  - 0001 — Tech stack (Rust + React + Cytoscape) confirmed
  - 0002 — Effect inference strategy (heuristic, per-language rules, confidence levels)
  - 0003 — Distribution strategy (GitHub Releases + npm wrapper + Homebrew tap)
  - 0004 — MCP tool API surface
- `docs/api.md` — full MCP tool schemas (request and response shapes)
- `docs/effect-rules/python.md` — Python effect detection rules in human-readable form
- `docs/effect-rules/typescript.md` — TS effect rules
- `docs/effect-rules/rust.md` — Rust effect rules
- A test fixture per language: `tests/fixtures/{python,typescript,rust}/sample-repo/` with deliberately varied effect patterns the agent will use to validate detection
- Wireframe sketch (text-based ASCII or markdown) of the three viewer modes in `docs/viewer-mockup.md`

**User decisions required:**
- Confirm tech stack (any objection to Rust + React + Cytoscape?)
- Confirm effect set (the 9 categories — add/remove?)
- Confirm MCP tool surface (4 tools — too many, too few, wrong shape?)
- Approve effect rules drafts (these define what the inference will catch)

---

## Checkpoint 2 — M1: Skeleton + Graph mode + Python parsing

**End of week 1.**

**Deliverables:**
- Full Rust workspace builds clean: `cargo build --release` and `cargo clippy --all-targets -- -D warnings`
- MCP server speaks the protocol; `tools/list` returns the 4 tools; `analyze_repo` works on a sample Python repo
- Tree-sitter integration for Python; AST → graph extraction works
- Frontend skeleton: Vite + React + Cytoscape rendering a graph from JSON
- HTTP viewer at `localhost:7000` serves the embedded frontend
- End-to-end smoke test: `loom-lens analyze tests/fixtures/python/sample-repo/` produces a viewer URL; opening it shows the graph
- GitHub release tag: `v0.1.0-alpha.1`
- README updated with current install-from-source instructions
- CI green on GitHub Actions

**User decisions required:**
- Spot-check the graph quality on a real Python repo (the user provides one if possible)
- Approve continuing to M2

---

## Checkpoint 3 — M2: TypeScript + Rust + Effects mode

**End of week 2.**

**Deliverables:**
- TypeScript and Rust parsers integrated; graph extraction works for both
- Effect inference engine implemented from the rules in `docs/effect-rules/`
- Effects mode in the viewer: nodes colored by inferred effects with confidence indicators
- Effect filters in the UI: "show only Net," "hide pure functions," etc.
- Hover state on a node shows the evidence ("Net inferred from `fetch()` at line 42")
- Tests: each effect rule has a fixture demonstrating detection (positive case) and a near-miss demonstrating non-detection (negative case)
- Tag: `v0.2.0-alpha.1`
- Updated README with effect rules summary

**User decisions required:**
- Run on a real codebase (the user's own, ideally) and assess effect inference accuracy
- Identify missed patterns or false positives — these become the rule corrections for M3
- Approve continuing to M3

---

## Checkpoint 4 — M3: Hashes + agent integration polish

**End of week 3.**

**Deliverables:**
- AST normalization implemented per language (whitespace, comments, imports stripped per ARCHITECTURE.md §6)
- BLAKE3 hashing of normalized ASTs
- Hash mode in the viewer: duplicate detection (identical hashes across files), near-duplicate detection (low edit distance over canonical form)
- Git history integration: when a repo is a git repo, `compare_hashes` between two refs identifies what actually changed (semantically, not textually)
- MCP tool descriptions tightened based on M2 feedback — agent should be able to use the tools effectively without explanation
- Structured output formats (not just viewer URLs) for programmatic agent consumption
- Tag: `v0.3.0-alpha.1`

**User decisions required:**
- Test on a real refactoring session: the user picks a recent commit in one of his repos, runs `compare_hashes` against the previous commit, judges whether the output is more useful than `git diff`
- Approve continuing to M4

---

## Checkpoint 5 — M4: Polish + Public release

**End of week 4.**

**Deliverables:**
- Documentation complete: `README.md` polished for public consumption, `docs/` covers usage and architecture, `CONTRIBUTING.md` is real
- Demo: an asciinema recording or short MP4 showing the three modes in action on a real codebase. Linked from README.
- Pre-built binaries for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin` produced by GitHub Actions on tag push
- npm wrapper package ready (does not auto-publish; awaits user approval)
- Homebrew tap repository created (`thefamoushesham/homebrew-tap`) with a `loom-lens.rb` formula
- Optional: blog post draft in `docs/blog-post.md`
- Optional: MCP registry submission prepared
- Tag: `v0.1.0` (no longer alpha)

**User decisions required (this is the big one):**
- Approve npm publish (`npm publish`)
- Approve crates.io publish (`cargo publish`)
- Approve Homebrew tap push
- Approve any blog post or marketing content before it goes live
- Decide whether to submit to the MCP registry now or wait for v0.2

---

## Always-interrupt conditions

Independent of checkpoints, the agent stops immediately and writes to `BLOCKED.md` if:

- A security rule would have to be violated.
- The agent is about to publish to a registry (always user-only).
- An external PR or issue requires a response.
- A toolchain or dependency is unobtainable.
- Token spend approaches budget cap.
- The agent finds itself making the same edit attempt more than 3 times.
- Any mode passes a milestone but only works for one language — multi-language is a hard requirement.
