# Architecture

Design decisions for Loom Lens. Treat this as the contract; deviations require an ADR.

---

## 1. What Loom Lens is

A single statically-linked Rust binary (`loom-lens`) that:

1. Speaks the **Model Context Protocol** over stdio for Claude Code integration.
2. Parses source repositories using **Tree-sitter** for any supported language.
3. Builds a **typed graph** of files, modules, functions, types, calls, and imports.
4. Performs **heuristic effect inference** per language using rule files in `docs/effect-rules/`.
5. Computes **content-addressed hashes** of normalized ASTs using BLAKE3.
6. Serves an **HTTP viewer** on `localhost:7000` with the React + Cytoscape frontend embedded.

The frontend is bundled into the Rust binary at build time via `rust-embed`. Distribution is therefore a single binary with no runtime dependencies beyond the OS.

---

## 2. Tech stack

### Backend (Rust)
- **MCP server:** the official Anthropic Rust MCP SDK (or `rmcp` if the official one isn't ready), stdio transport.
- **HTTP server:** `axum` for the viewer, `tower` middleware for static file serving.
- **Parsing:** `tree-sitter` + per-language grammars (`tree-sitter-python`, `tree-sitter-typescript`, `tree-sitter-rust`).
- **Graph:** `petgraph` for in-memory graph operations.
- **Hashing:** `blake3`.
- **Async:** `tokio`.
- **Serialization:** `serde` + `serde_json`.
- **CLI:** `clap`.
- **Embedding frontend:** `rust-embed`.
- **Logging:** `tracing` + `tracing-subscriber`.

### Frontend (TypeScript + React)
- **Build:** Vite.
- **UI:** React (functional components, hooks).
- **Styling:** Tailwind CSS.
- **Graph rendering:** Cytoscape.js (via `react-cytoscapejs`).
- **State:** Zustand (lighter than Redux for this scale).
- **Routing:** React Router.

### Why Rust for the backend
- Single statically-linked binary; trivial to distribute.
- Tree-sitter has excellent Rust bindings.
- BLAKE3 is native and fast.
- Same language as Loom main project — code can be shared if patterns work out.
- Strong typing matches the project's thesis.

### Why React + Cytoscape for the frontend
- Cytoscape handles 10k+ node graphs without lag (D3 forces you to build layout from scratch).
- React's component model fits the three-mode UX (Graph / Effects / Hashes share one graph; the modes are visualization layers).
- Tailwind keeps styling out of the way.
- Vite's dev server has fast HMR for iteration speed.

---

## 3. Crate layout

```
crates/
├── core/        # Tree-sitter integration, graph extraction, language traits
├── effects/     # Effect inference engine and per-language rules
├── hashing/     # AST normalization and BLAKE3 hashing
├── mcp/         # MCP server and tool definitions
├── viewer/      # HTTP server and frontend asset embedding
└── cli/         # Binary entry point; subcommands: serve, analyze
```

Workspace root `Cargo.toml` shares dependencies via `[workspace.dependencies]`. Each crate is small and focused; the binary is in `cli`.

---

## 4. The graph data model

```rust
pub struct CodeGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub edges: Vec<Edge>,
    pub languages: Vec<Language>,
    pub repo_root: PathBuf,
    pub generated_at: DateTime<Utc>,
}

pub enum Node {
    File { path: PathBuf, language: Language, lines: usize },
    Module { name: String, file: NodeId },
    Function { name: String, signature: String, span: Span, effects: EffectSet, hash: Hash },
    Type { name: String, kind: TypeKind, span: Span, hash: Hash },
}

pub enum Edge {
    Contains { parent: NodeId, child: NodeId },     // file→module, module→function
    Calls { caller: NodeId, callee: NodeId, sites: Vec<Span> },
    Imports { from: NodeId, to: NodeId },           // module→module
    References { from: NodeId, to: NodeId },        // function→type, etc.
}
```

The graph is the lowest common denominator across the three modes. Each mode is a visualization layer reading the same graph.

---

## 5. Effect inference

Effect inference is **heuristic**, **per-language**, and **explicit about uncertainty**.

### Effect set
- `Pure` (default; absence of evidence)
- `Net` — network calls
- `IO` — filesystem, stdout/stderr beyond debug logs
- `Mut` — non-local mutation
- `Throw` — uncaught exceptions / panics / explicit throws
- `Async` — async runtime usage
- `Random` — non-deterministic randomness
- `Time` — clock reads, sleeps
- `Foreign` — FFI / unsafe / native binding

### Inference strategy
For each function in the parsed AST, walk the body and look for known patterns from the language's effect rules file. Patterns are AST-shape matchers, not regex. Each match assigns an effect with a confidence: `definite`, `probable`, `possible`.

Effects propagate transitively through the call graph (intra-repo only — calls into external libraries are tagged `External` rather than expanded).

### Confidence in the UI
- `definite` → solid color
- `probable` → striped pattern
- `possible` → outline only

The user sees uncertainty visually. Hovering shows the evidence ("`fetch()` call at line 42").

### Per-language rules
Effect rules live in `docs/effect-rules/{python,typescript,rust}.md` as human-readable documents that the agent translates into AST-matcher code at M2.

---

## 6. Hashing

Hash identity is BLAKE3 of the normalized AST. Normalization rules:

- Whitespace and comments stripped.
- Identifier names preserved (renaming changes the hash, intentionally — at least for v1; this is a design decision to revisit).
- Literal values preserved (`42` and `41` produce different hashes).
- Imports/use-statements stripped from function bodies (they affect compilation, not behavior).

Hash uses: dedup detection, change tracking, cross-file identity recognition.

The hash space and Loom main's hash space are intentionally compatible — same algorithm, same normalization rules — so Loom Lens hashes can be reused later if Loom main proceeds.

---

## 7. MCP tool API

The MCP server exposes tools to Claude Code. See `docs/api.md` for full schemas. Summary:

| Tool | Purpose |
|------|---------|
| `analyze_repo` | Parse a repo, return a viewer URL and a summary |
| `query_graph` | Structured queries: "all functions with `Net` effect," "all duplicates of function X," etc. |
| `get_function_context` | Return the source + immediate callers/callees of a named function |
| `compare_hashes` | Given two commit refs, return changed/unchanged hashes |

All tools return JSON with structured fields plus a `viewer_url` where appropriate.

---

## 8. Distribution

- **Primary:** GitHub Releases with pre-built binaries for `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-apple-darwin`. Windows deferred to v0.2 unless someone asks.
- **npm wrapper:** `loom-lens` package that downloads the appropriate binary from GitHub Releases on `npm install -g`. Pattern from `esbuild`/`swc`.
- **Cargo:** `cargo install loom-lens` builds from source. Slower but works for any platform with a Rust toolchain.
- **Homebrew tap:** `brew tap thefamoushesham/tap && brew install loom-lens`. Set up at M4.

The npm wrapper is the recommended install path because most Claude Code users have Node already.

---

## 9. What's deferred

- **Windows support.** Deferred to v0.2 unless real demand. Not a strategic decision; just scope cut.
- **More languages.** Adding Go, Java, Ruby, etc., is small per-language work but slows down v0.1. Defer to community contributions.
- **Live updates.** v0.1 is "analyze and view"; the graph is static after `analyze_repo`. Watch-mode with incremental re-parse is v0.2.
- **Graph editing.** Read-only forever. Edits go through the user's editor.
- **Custom layout algorithms.** Cytoscape's defaults are good enough; custom layouts are a v0.3 polish item.
- **Effect inference soundness.** Heuristic always. A sound effect system is what Loom main is for.

---

## 10. What this teaches us about Loom

For each mode, the question Loom Lens is set up to answer:

- **Graph mode:** Is graph visualization useful enough that humans navigate by it instead of the file tree? *Evidence comes from usage telemetry (opt-in) and direct user feedback.*
- **Effects mode:** Does seeing effects make codebases easier to reason about? Is the inference accurate enough to trust? *Evidence comes from comparison with codebases the user knows well — does the lens surprise them?*
- **Hash mode:** Does dedup detection actually find duplicates worth deduping? Does identity-tracking-across-renames help? *Evidence comes from real refactoring sessions.*

A successful Loom Lens informs Loom main's design priorities. A failed Loom Lens — features people don't use, inference people don't trust — is a strong signal to rethink the language thesis before committing months to it.
