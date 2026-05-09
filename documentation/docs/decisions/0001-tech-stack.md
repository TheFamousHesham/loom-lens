# 0001 — Tech Stack: Rust Backend, React+Cytoscape Frontend

**Date:** Pre-Checkpoint 1
**Status:** Accepted (2026-05-09)
**Reviewed at checkpoint:** 1 ✓

## Context

Loom Lens has three runtime components: an MCP server (talks to Claude Code over stdio), a parser/analyzer (heavy work, uses Tree-sitter, may run on large repos), and an HTTP server (serves the viewer to the browser). All three can ship as a single binary or as separate processes.

Constraints:
- Must distribute easily — `npm install -g`, `cargo install`, or `brew install`.
- Must handle 10k+ node graphs without melting.
- Must be fast to iterate on (this is a 4-week project).
- Should reuse parser/hashing/effect code if Loom main proceeds.

## Decision

**Single Rust binary for backend; React + Cytoscape.js for frontend, embedded into the binary at build time.**

- Backend Rust crates: `core` (Tree-sitter + graph extraction), `effects` (effect inference), `hashing` (BLAKE3 + AST normalization), `mcp` (MCP server), `viewer` (HTTP server), `cli` (binary entry point).
- Frontend: Vite + React + TypeScript + Tailwind + Cytoscape.js + Zustand for state.
- Frontend bundled into the Rust binary via `rust-embed`.
- Distribution: GitHub Releases binaries, npm wrapper that downloads them, optional Homebrew tap.

## Alternatives considered

- **Pure Node/TypeScript backend:** simpler MCP integration (the official SDK is most polished in TS), faster iteration. Rejected because: (a) Tree-sitter is much faster from Rust, (b) BLAKE3 is native in Rust, (c) reusing code with Loom main matters, (d) single-binary distribution is much easier from Rust.
- **Python backend:** even simpler MCP integration, but slow for the parser-heavy workload, and distribution is painful (pip dependencies, no single binary).
- **Two-process architecture (Rust analyzer + Node MCP server):** more flexible but adds operational complexity for a 4-week project. Rejected.
- **Different graph rendering (D3, vis.js, Sigma.js, Reagraph):** D3 forces you to build layout from scratch; vis.js is showing its age; Sigma.js is fast but smaller ecosystem; Reagraph is React-native but newer/less battle-tested. Cytoscape is the safe choice with the best 10k-node story.
- **No frontend (TUI only):** rejected. The user picked Mode 1 (Graph view) which is fundamentally a visual artifact.
- **Electron app:** rejected. Browser-based viewer is good enough and avoids 100MB of Chromium runtime.

## Consequences

### Positive
- Single statically-linked binary, ~30 MB, no runtime dependencies.
- Tree-sitter parsing is fast.
- Code can be shared with Loom main if the bet pays off.
- React + Cytoscape is a well-trodden path; lots of references.
- Frontend hot-reload during development via Vite.

### Negative
- Two languages to maintain (Rust + TS). Frontend devs can't easily contribute to backend and vice versa.
- MCP SDK in Rust is less mature than the TS one. We may need to fall back to community crates.
- Compile times for the Rust workspace will be 30-60s on a clean build. Fine with sccache; painful on CI.

### Risks
- The official Rust MCP SDK may not exist or may have rough edges. Mitigation: use `rmcp` (community) initially; switch to official when ready. The protocol is JSON-RPC over stdio — implementable from scratch in a day if both options fall through.
- Cytoscape's WebGL renderer has occasional bugs at very large graphs. Mitigation: cap at 10k visible nodes with a "graph too large, please filter" UX.

## References

- Cytoscape.js: https://js.cytoscape.org/
- rust-embed: https://crates.io/crates/rust-embed
- Tree-sitter Rust bindings: https://crates.io/crates/tree-sitter
- Anthropic MCP spec: https://modelcontextprotocol.io/

## Refinements at Checkpoint 1

The original draft is sound; no part of the decision is reversed. The following clarifications were resolved or surfaced during the Checkpoint 1 review of cross-document consistency:

- **Toolchain pins are now concrete.** `mise.toml` pins Rust 1.85.0, Node 22.11.0, pnpm 9.15.0, plus `cargo-nextest`, `cargo-audit`, `cargo-deny`, `cargo-watch`, `sccache`. Rust 1.85.0 ships the 2024 edition; we adopt it for new crates. Bumping any of these is itself an architectural change and warrants a new ADR or an amendment here.
- **Cargo-tool MSRV coupling, recorded 2026-05-09.** `cargo-deny` and `cargo-nextest`'s latest releases require rustc ≥ 1.88, which conflicts with our 1.85.0 pin. Both are therefore pinned to specific older versions (`cargo-deny = "0.16.4"`, `cargo-nextest = "0.9.95"`) rather than `"latest"` in `mise.toml`. The other three (`cargo-audit`, `cargo-watch`, `sccache`) track `"latest"` because their MSRV stays below 1.85.0. When we bump Rust (in a future ADR), revisit this pinning and consider unpinning the three back to `"latest"` to track upstream security updates.
- **MCP Rust SDK choice is deferred to M1 implementation.** The ADR's risk note still applies. M1 will pick between (a) the official Anthropic Rust MCP SDK if available and stable, (b) the community `rmcp` crate, or (c) a hand-rolled JSON-RPC-over-stdio implementation. Whichever is chosen, the choice will be recorded as an amendment to this ADR rather than as a new ADR — it's an implementation detail of the same decision.
- **Cytoscape version family.** We target Cytoscape.js 3.x (the current major). The performance budget — render 10k visible nodes without dropped frames on a mid-tier laptop — is a M2 acceptance criterion; if Cytoscape can't hit it, we revisit (Reagraph or Sigma.js) and supersede this ADR. Until then, Cytoscape remains the choice.
- **Frontend embedding mechanics.** The frontend is built into `frontend/dist/` and embedded via `rust-embed` into the `viewer` crate. `frontend/dist/` is gitignored; the binary build pipeline (and the release workflow) runs `pnpm build` before `cargo build --release`. CI builds confirm this every push.
- **Multi-language repos.** The parser is invoked per-file; a repo with Python *and* TypeScript produces a single graph with nodes from both languages and inter-language calls only when they cross via FFI/IPC (which we tag `Foreign`). No design change needed; called out here so it isn't relitigated.
