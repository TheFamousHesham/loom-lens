# Loom Lens

> See your codebase as a graph: structure, effects, and content-addressed identity.

Loom Lens is a Claude Code plugin (MCP server) plus a browser-based viewer that visualizes any codebase three ways:

- **Graph** — files, modules, functions, types, and the relationships between them.
- **Effects** — every function colored by its inferred side effects: network, IO, mutation, throws, randomness, time. Filter by effect to find every place your code touches the network.
- **Hashes** — content-addressed function identity. Surface duplicates, track logical identity across renames, see which functions a change actually affects.

Loom Lens supports Python, TypeScript/JavaScript, and Rust at launch.

> **Status: pre-alpha.** This README describes the project's intent. Working code lands at Milestone 1. See `STATUS.md` for current state.

---

## Why

Code is a graph that humans read as a stream of text. Most of what makes a codebase hard to navigate is graph-shaped: which functions hit the network, which modules form a dependency cycle, which "different" functions are actually the same. Loom Lens makes the graph visible.

The project is also a research probe for a larger language design called Loom. Loom hypothesizes that graph-as-canonical-source, explicit effects, and content addressing produce measurably better outcomes for AI-driven software development. Loom Lens tests that hypothesis on existing codebases before any new language is introduced. If the lens is useful, the hypothesis is supported. If it's not, we know early.

---

## Install

> Not yet published. Install instructions land at v0.1.0 (Milestone 4). Until then, build from source:

```bash
git clone https://github.com/TheFamousHesham/loom-lens
cd loom-lens
mise install        # installs Rust, Node, etc.
cargo build --release
cd frontend && pnpm install && pnpm build
```

When v0.1.0 ships:

```bash
npm install -g loom-lens
# or
brew install thefamoushesham/tap/loom-lens
```

---

## Use

Add to your Claude Code MCP config:

```json
{
  "mcpServers": {
    "loom-lens": {
      "command": "loom-lens",
      "args": ["serve"]
    }
  }
}
```

In Claude Code:

> "Show me the effects in this codebase."

Claude calls the `analyze_repo` tool. The MCP server parses the repo, returns a viewer URL. Open it in your browser. Three modes available in the top bar.

You can also use it standalone without Claude Code:

```bash
cd /path/to/your/repo
loom-lens analyze .
# Open http://localhost:7000 in your browser
```

---

## What it does (and doesn't)

**Does:**
- Parse Python, TypeScript, and Rust codebases.
- Build a graph of files, modules, functions, types, calls, imports.
- Identify likely side effects per function using language-specific heuristics.
- Compute BLAKE3 hashes of normalized ASTs for duplicate detection and identity tracking.
- Render an interactive graph in the browser (Cytoscape.js, scales to ~10k nodes).
- Expose all of this to Claude Code as MCP tools for programmatic access.

**Does not:**
- Provide *sound* effect analysis. Effects are heuristic and can have false positives or negatives. Treat as a guide, not a guarantee.
- Edit code. Loom Lens is read-only — open the linked file in your editor to make changes.
- Replace your IDE or LSP. It's a complementary lens.
- Work on languages other than Python, TypeScript, and Rust at launch. Adding more is straightforward (Tree-sitter does the heavy lifting); see `docs/effect-rules/` for the per-language file structure.

---

## Architecture

A single Rust binary that:
1. Speaks the MCP protocol over stdio (Claude Code integration).
2. Parses repos via Tree-sitter.
3. Serves an HTTP viewer on `localhost:7000`.
4. Embeds the React/Cytoscape frontend.

See `ARCHITECTURE.md` for design rationale.

---

## Roadmap

| Milestone | Target | What's in it |
|-----------|--------|--------------|
| **M1** | Week 1 | Skeleton + Graph mode, Python parsing |
| **M2** | Week 2 | TypeScript and Rust parsing, Effects mode |
| **M3** | Week 3 | Hash mode, Git history, polished agent integration |
| **M4** | Week 4 | Docs, demo, npm/cargo publish, public release |

See `CHECKPOINTS.md` for the detailed checkpoint list.

---

## Contributing

The project is being built largely by Claude Code working autonomously between checkpoints. External contributions are welcome but please open a discussion first — the development cadence is unusual and we want to avoid stepping on the agent's work.

See `CONTRIBUTING.md`.

---

## License

MIT. See `LICENSE`.

---

## Author

Hesham Mashhour, MD ([@TheFamousHesham](https://github.com/TheFamousHesham)). Better Brain Lab LLC.

Built as the analytical surface of a larger language research project. Both projects are pursuing the question: *what does software look like if it's designed for an AI to write and a human to verify, instead of the other way around?*
