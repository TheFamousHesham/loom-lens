# 0004 — MCP Tool API Surface

**Date:** Pre-Checkpoint 1
**Status:** Accepted (2026-05-09)
**Reviewed at checkpoint:** 1 ✓

## Context

Loom Lens is consumed by Claude Code (and other MCP clients) through a small set of tools. Tool design affects how naturally the agent uses the lens — too many tools confuses the model; too few forces it to make many small calls; the wrong shape forces awkward workflows.

The agent's likely use cases:
- "Show me this codebase" — broad orientation.
- "Find every place we hit the network" — targeted query.
- "Why is `parseUser` calling `validateEmail`?" — focused exploration.
- "What changed semantically between commit A and commit B?" — diff understanding.

## Decision

**Four tools.** Each returns structured data plus, where appropriate, a viewer URL for human inspection.

### `analyze_repo`
Parses a repo and returns a graph summary plus a viewer URL.

**Input:**
```json
{
  "path": "/path/to/repo",
  "languages": ["python", "typescript", "rust"],
  "include_external_calls": false
}
```

**Output:**
```json
{
  "viewer_url": "http://localhost:7000/r/abc123",
  "summary": {
    "files": 142,
    "functions": 1830,
    "languages": {"python": 87, "typescript": 55},
    "effect_distribution": {"Pure": 1240, "Net": 145, "IO": 88, ...}
  },
  "graph_id": "abc123"
}
```

### `query_graph`
Structured queries against an analyzed repo.

**Input:**
```json
{
  "graph_id": "abc123",
  "query": {
    "kind": "functions_with_effect",
    "effect": "Net",
    "min_confidence": "probable"
  }
}
```

Query kinds: `functions_with_effect`, `duplicates_of`, `callers_of`, `callees_of`, `dependents_of_file`, `cycle_detection`, `unreachable_functions`.

**Output:**
```json
{
  "results": [
    {
      "node_id": "fn_142",
      "name": "fetch_user",
      "file": "src/api/users.py",
      "line": 42,
      "effects": ["Net", "Throw"],
      "evidence": [...]
    }
  ],
  "viewer_url": "http://localhost:7000/r/abc123?filter=effect:Net"
}
```

### `get_function_context`
Returns source code plus immediate callers/callees of a named function.

**Input:**
```json
{
  "graph_id": "abc123",
  "function": "src/api/users.py::fetch_user"
}
```

**Output:**
```json
{
  "function": {
    "name": "fetch_user",
    "signature": "def fetch_user(user_id: int) -> User",
    "source": "...",
    "effects": ["Net", "Throw"],
    "hash": "blake3:7f3a..."
  },
  "callers": [...],
  "callees": [...]
}
```

### `compare_hashes`
Given two refs (commits, branches, or working tree), returns which functions changed semantically.

**Input:**
```json
{
  "path": "/path/to/repo",
  "before": "HEAD~1",
  "after": "HEAD"
}
```

**Output:**
```json
{
  "changed": [
    {
      "function": "src/api/users.py::fetch_user",
      "before_hash": "blake3:7f3a...",
      "after_hash": "blake3:9d4a...",
      "kind": "modified"
    }
  ],
  "added": [...],
  "removed": [...],
  "renamed": [
    {"from": "old_name", "to": "new_name", "hash": "blake3:..."}
  ],
  "viewer_url": "http://localhost:7000/r/abc123/diff?before=...&after=..."
}
```

## Alternatives considered

- **One mega-tool with a query DSL.** Rejected. Tool-per-purpose lets the agent reason about what's available; one mega-tool obscures the surface.
- **Many small tools** (`find_callers`, `find_callees`, `find_imports`, `find_types_referenced_by`, etc.). Rejected. Inflates the tool count, makes the agent indecisive about which to call. `query_graph` with kinds is cleaner.
- **No structured output, viewer URL only.** Rejected. The agent needs structured data to reason; humans need the URL. Both serve real users.
- **Streaming results.** Deferred. v0.1 returns full results; streaming is a v0.2 optimization.

## Consequences

### Positive
- Four tools is a manageable surface for the agent.
- Each tool has a clear "I would use this when…" answer.
- Structured output enables programmatic use; URLs enable human inspection.
- Query kinds are extensible without breaking the API surface.

### Negative
- Adding a new query kind requires both backend changes and (often) frontend filter changes.
- Queries that don't fit the kinds (e.g., custom graph algorithms) require multiple `query_graph` calls.

### Risks
- Tool descriptions may not be self-explanatory enough; the agent may underuse `compare_hashes` because it sounds esoteric. Mitigation: write good tool descriptions during M3 polish; iterate based on observation.
- A 5th tool will be tempting at M3. Resist unless there's a use case the existing four can't cover.

## References

- MCP tool spec: https://modelcontextprotocol.io/specification#tools
- Anthropic guidance on tool design: https://docs.anthropic.com/en/docs/build-with-claude/tool-use

## Refinements at Checkpoint 1

The four-tool surface stands. Refinements concern the *shape* of inputs/outputs, error semantics, and the `graph_id` lifecycle — implementation details that need locking before M2 wires the agent against this contract.

- **`graph_id` lifecycle, locked.** A `graph_id` returned by `analyze_repo` is a content-addressed handle: `blake3(canonical_repo_root_path || sorted_file_blake3s)`, truncated to 12 hex chars for readability. Re-analyzing the same repo at the same commit produces the same `graph_id`. A modified repo produces a different `graph_id`. This is idempotent and cacheable. The viewer holds graphs in memory, with an LRU eviction at 8 graphs (~1 GB headroom). `query_graph` against an evicted `graph_id` returns error `1001` and the agent reanalyzes.
- **Per-tool "when not to use" descriptions.** Tool descriptions in the MCP server include both "use when …" *and* "do not use when …" hints, since negative guidance reduces miscalls more than positive guidance alone. Concrete suggested wording lives in `documentation/docs/api.md`.
- **`query_graph` query kinds, locked.** The seven kinds (`functions_with_effect`, `duplicates_of`, `callers_of`, `callees_of`, `dependents_of_file`, `cycle_detection`, `unreachable_functions`) cover the use cases enumerated in Context. Adding a new kind is non-breaking (it appears in the enum); removing one is breaking (major version bump). `min_confidence` defaults to `possible` (most permissive) since the agent typically wants the broadest results and filters client-side.
- **`compare_hashes` working-tree semantics.** The literal string `"WORKING_TREE"` is reserved as a magic ref meaning "uncommitted state on disk." This is the only non-git ref. All other strings are passed to `git rev-parse` and must resolve to a commit.
- **Error code reserved range.** Error codes `1000-1099` are reserved for analysis errors (repo/path issues), `2000-2099` for graph-query errors, `3000-3099` for git/version errors. Within those bands, codes are stable across versions (we don't reuse a retired code for a new meaning).
- **Streaming deferred, but confirmed plausible.** v0.1 returns full results. Streaming (server-sent JSON-RPC notifications) is a v0.2 optimization for `query_graph` results > 1000 entries. The API surface today does not foreclose it.
- **Result size cap.** `query_graph` results are capped at 5000 entries by default with a `limit` parameter to override. Exceeding the cap returns the first `limit` entries plus a `truncated: true` flag, never silently discards.
- **No fifth tool.** The temptation at M3 to add (e.g.) `summarize_module` or `find_pattern` is real and should be resisted unless the existing four genuinely cannot serve the use case. If a fifth is genuinely needed, it gets its own ADR.
