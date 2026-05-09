# Status

Updated at end of every session and at every checkpoint by the agent. Read this at the start of every session before doing anything else.

---

## Last updated

**Timestamp:** 2026-05-09
**Session:** 1 (continuing)
**Agent:** Claude (Opus 4.7, 1M context)
**Current commit:** advancing through M1; see `git log`

## Current phase

**Checkpoint 2 — M1: Skeleton + Graph mode + Python parsing — reached.**

Checkpoint 1 closed: user said "Go" on 2026-05-09 after reviewing the bootstrap and design deliverables. ADRs 0001–0004 moved Proposed → Accepted; the four-tool MCP surface, 9-effect taxonomy, "`Result` is not an effect" decision, and tech-stack/distribution choices are locked.

M1 deliverables landed across three commits (workspace skeleton, parser + analyze_repo wiring, and this checkpoint's frontend + tag). `cargo build --release` and `cargo clippy --all-targets -- -D warnings` are clean against rustc 1.85.0; `cargo test --workspace` passes (parser integration test against the Python fixture). End-to-end smoke confirmed: `loom-lens analyze tests/fixtures/python/sample-repo/` prints a working viewer URL; the served SPA fetches `/api/graph/<id>` and renders the structural graph in Cytoscape; `analyze_repo` over MCP stdio returns `graph_id`, `viewer_url`, and a populated summary.

## What's done

### Reorganization

- Extracted the kickoff tarball into the project root and adopted the two-tier layout described in ADR 0005 (`documentation/docs/decisions/0005-repo-layout.md`):
  - State, public surface, operational config, and infrastructure live at the repo root.
  - `ARCHITECTURE.md`, `CHECKPOINTS.md`, and the `docs/` design library live under `documentation/`.
- Updated cross-references in `CLAUDE.md`, `README.md`, `CONTRIBUTING.md` to reflect the new paths.
- Updated `scripts/init.sh`, `scripts/run-agent.sh`, `scripts/snapshot.sh`, `scripts/migrate.sh` to resolve `LOOM_LENS_ROOT` from the script's own location by default. They keep working in `/opt/loom-lens` (production) and now also work in `/home/cc/ProjectAlpha` (current dev environment) without an explicit env export.
- Aligned `.env.example` and `CLAUDE.md` to the user's actual git identity (`hesham@betterbrainlab.org`).
- Renamed the default branch from `master` to `main` (per the agent's Git discipline rules: "Never push --force on `main`" assumes the branch is `main`).
- Installed the `commit-msg` (conventional-commits enforcer) and `pre-commit` (secret detector) hooks verbatim from `scripts/init.sh`.
- Added `origin` remote pointing at `git@github.com:TheFamousHesham/loom-lens.git`. Reachability via SSH confirmed; the repository itself does not yet exist on GitHub — see BLOCKED.md.

### Checkpoint 1 deliverables (per `documentation/CHECKPOINTS.md`)

- ADRs reviewed and refined:
  - `0001-tech-stack.md` — refinements added at "Refinements at Checkpoint 1": toolchain pins concrete, MCP SDK choice deferred to M1 implementation, Cytoscape 3.x version family, frontend embedding mechanics, multi-language repo handling. Status remains **Proposed**.
  - `0002-effect-inference.md` — refinements added: 9-effect set locked, `Result`/`Option` is *not* an effect, effect aggregation rule (union + confidence weakening on transitive inheritance), SCC fixpoint for recursion, `External` as a separate provenance tag, confidence→UI mapping locked, normative status of effect-rule files. Status remains **Proposed**.
  - `0003-distribution.md` — refinements added: explicit conflict surfaced between auto-name-reservation and "never publish without approval"; resolution to defer registration; Linux ARM64 added to release matrix; SHA-256 checksums for binaries; Sigstore signing deferred to v0.2; Homebrew tap repo creation deferred to user approval. Status remains **Proposed**.
  - `0004-mcp-tool-api.md` — refinements added: `graph_id` lifecycle locked (content-addressed, 12-hex, LRU); per-tool "do not use when …" descriptions; `query_graph` query kinds locked; `compare_hashes` `WORKING_TREE` magic ref reserved; error-code reserved bands; result-size cap with `truncated` flag; "no fifth tool" guardrail. Status remains **Proposed**.
- New ADR filed: `0005-repo-layout.md`. Status: **Accepted** (the action it describes was executed in this session).
- `documentation/docs/api.md` polished into a full implementation contract: per-tool input schemas as JSON Schema with `oneOf` per query kind, complete output shapes, error-code envelope, HTTP-only endpoint table, versioning policy.
- `documentation/docs/effect-rules/python.md` extended: added `asyncpg`, `aiokafka`, `asyncssh`, `grpc`/`grpcio` to Net; stdin reading (`input`, `sys.stdin.*`) and disk-backed serialization round-trips to IO; `assert` statement to Throw.
- `documentation/docs/effect-rules/typescript.md` extended: added tRPC, GraphQL clients, server-side framework data fetchers, gRPC to Net; File System Access API and OPFS to IO.
- `documentation/docs/effect-rules/rust.md` extended: added `tonic`, `tarpc`, `grpcio` to Net; resolved the open "Decision pending" by deferring to ADR 0002's refinement (`Result` is not an effect).
- Test fixtures created at `tests/fixtures/{python,typescript,rust}/sample-repo/`. Each fixture is a small parseable codebase with one module per effect category, plus pure controls and false-positive lures. Every function has an `# expect: …` / `// expect: …` annotation declaring the inference engine's expected output. Build status: Python and TypeScript fixtures are syntactically valid; Rust fixture deliberately does not compile (it references `reqwest`, `tokio`, etc. without declaring deps — Tree-sitter parses the source).
  - Two effect categories in the TypeScript Foreign fixture were trimmed (a `child_process` invocation and a dynamic-code-evaluation constructor demo) because the dev environment's safety hooks reject the literal source. The omission is documented in-file; the rules and Python fixture cover the same patterns.
- `documentation/docs/viewer-mockup.md` — text-based wireframes for the three modes plus common chrome and cross-mode interactions. Locks the layout, color palette (with WCAG-AA-conscious choices and redundant patterns for colorblind users), confidence-fill mapping, and selection persistence.

## What's done at M1

- **Cargo workspace** with the six crates from ADR 0001 (`core`, `effects`, `hashing`, `mcp`, `viewer`, `cli`). `cargo build --release` produces a single ~6 MB statically-linked binary at `target/release/loom-lens` with the React/Cytoscape SPA embedded via `rust-embed`.
- **Tree-sitter Python parser** in `crates/core` walking `function_definition`, `class_definition`, `decorated_definition`, `call`, `import_statement`, and `import_from_statement` AST nodes. Two-pass extraction: per-file walks emit nodes + Contains edges + pending call/import records; a name-index resolution pass turns pending records into Calls and Imports edges across modules.
- **MCP server** in `crates/mcp` (hand-rolled JSON-RPC 2.0 over stdio, ~250 LOC including envelopes, dispatch, and four tool descriptors mirroring `documentation/docs/api.md`). `initialize` and `tools/list` work; `tools/call analyze_repo` runs the parser on a `spawn_blocking` task and returns `{ graph_id, viewer_url, summary }`. `query_graph`, `get_function_context`, and `compare_hashes` return `MethodNotFound` until M2/M3.
- **HTTP viewer** (axum 0.7) at `crates/viewer` serving `/healthz`, `/r/:id` (SPA), `/_loom/assets/*` (rust-embedded assets), `/api/graph/:id` (graph JSON), `/api/graph/:id/source/*file` (source files in the graph's repo root). LRU graph cache (8 entries, per ADR 0004).
- **Vite + React + Cytoscape.js + Zustand** frontend at `frontend/`. Dark-themed shell with three-mode tabs (only Graph active at M1; Effects and Hashes show as disabled with M2/M3 tooltips). Renders nodes by kind (file/module/function/type) with language-coloured fills and edges differentiated by kind (calls solid blue, imports dotted, references dashed). Selection highlights and a footer reflecting parse stats.
- **CLI** in `crates/cli` with two subcommands: `serve` (MCP stdio + HTTP viewer side-by-side) and `analyze <path>` (one-shot parse, populate cache, print URL, hold the HTTP server until Ctrl-C; `--no-serve` exits after printing for machine consumption).
- **Tests**: `cargo test --workspace` passes; `crates/core/tests/python_fixture.rs` validates file/function counts, `fetch_user` resolution, and `graph_id` stability against `tests/fixtures/python/sample-repo/`.
- **`time` pinned to `=0.3.36`** (newer requires rustc 1.88+); **`cargo-deny` and `cargo-nextest` pinned** to versions compatible with Rust 1.85.0 (recorded in ADR 0001).
- **README** install-from-source steps reordered so `pnpm build` precedes `cargo build` (rust-embed needs the SPA bundle to exist at compile time).

## What's in progress

Nothing. M1 reached.

## What's next

User review per Checkpoint 2 in `documentation/CHECKPOINTS.md`. The user is asked to:

1. Spot-check the graph quality on a real Python repo of the user's choice — `loom-lens analyze /path/to/repo` is enough; the structural shape, function discovery, and call/import edges are what matters at M1.
2. Approve continuing to M2: TypeScript and Rust parsing, plus the Effects mode based on the rules in `documentation/docs/effect-rules/`.

After review, M2 work begins. The next ADRs (in addition to refining the rules per real-repo testing) will likely cover:
- TS/Rust parser dispatch (per-language strategy traits in `crates/core`).
- Effect inference engine architecture (rule registry, evidence carrying, transitive propagation per ADR 0002).
- Cross-attribute call resolution improvements for Python (currently only the rightmost segment of `client.get(...)` is matched).
3. Confirm the 9-effect taxonomy and the "Result is not an effect" decision in ADR 0002.
4. Confirm the four-tool MCP surface and the locked-down shapes in `documentation/docs/api.md`.
5. Spot-check the effect-rule documents — false negatives (patterns we should detect but missed) become rule corrections; false positives (patterns we shouldn't flag) become exclusion rules.
6. Spot-check the test fixtures' `# expect:` annotations against the effect-rule documents — they are the testable specification of correctness.
7. Decide on the open items in `BLOCKED.md`, especially the GitHub repository creation question.

After review and approval, M1 (Checkpoint 2) work begins: implementing the Rust workspace, Tree-sitter integration for Python, the MCP server protocol, and the React/Vite/Cytoscape frontend skeleton.

## Open decisions

(For Checkpoint 1 review by user)

- **Tech stack confirmation.** Rust + React + Cytoscape, with the toolchain pins in `mise.toml` (Rust 1.85.0, Node 22.11.0, pnpm 9.15.0). Any objection?
- **Effect set.** 9 categories: `Pure`, `Net`, `IO`, `Mut`, `Throw`, `Async`, `Random`, `Time`, `Foreign`. Add or remove?
- **Result/Option taxonomy.** ADR 0002 refinement says `Result<T, E>`-returning is *not* an effect; only panic-paths are tagged `Throw`. Confirm?
- **MCP tool surface.** Four tools with the shapes locked in `documentation/docs/api.md`. Object?
- **Repo creation.** GitHub repository at `TheFamousHesham/loom-lens` does not yet exist. Decide: user creates manually, or authorizes the agent to `gh repo create`, or defers push.
- **crates.io / npm name reservation.** ADR 0003 refinement defers this to user approval. The original "register at Checkpoint 1" was a conflict with the security rule.
- **Anthropic API key spend cap.** Confirm a hard cap is set in the Anthropic console.

## Token spend this session

Not instrumented at this granularity. The session bootstrapped the project, read all design docs in full, reorganized the layout, refined four ADRs, wrote one new ADR, polished the API spec, extended three effect-rule documents, built three test-fixture sample repos, and authored the viewer mockup. Roughly ~500 KB of documentation read; ~120 KB written.

## Notes

- Environment is a development workstation, not the production VPS the kickoff was written against. `mise` is not installed, `.env` does not exist, the nftables egress allowlist is not active. None of these block Checkpoint 1 (design only). All are documented in `BLOCKED.md` and need attention before M1.
- The kickoff authorized sudo for the initial `chown` of `/home/cc/ProjectAlpha` from root to `cc:cc`. No further use of sudo. The agent re-confirms: per CLAUDE.md §6, sudo is otherwise prohibited.
- The agent did **not** push to GitHub in this session because the remote repository does not yet exist. The next push is gated on the user's decision in `BLOCKED.md`.
- The agent did **not** publish to npm or crates.io. No registry actions were taken.
