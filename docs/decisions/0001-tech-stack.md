# 0001 — Tech Stack: Rust Backend, React+Cytoscape Frontend

**Date:** Pre-Checkpoint 1
**Status:** Proposed
**Reviewed at checkpoint:** 1 (pending)

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
