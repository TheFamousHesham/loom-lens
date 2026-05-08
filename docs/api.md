# MCP Tool API

Loom Lens exposes four tools to Claude Code (and other MCP clients) over stdio JSON-RPC. Full schemas in this document; design rationale in `docs/decisions/0004-mcp-tool-api.md`.

> **Status:** Specified at Checkpoint 1; implemented at Checkpoint 2; refined at Checkpoint 4. This document is the source of truth — when behavior diverges, fix the implementation, not the spec.

---

## Tools

### `analyze_repo`

Parses a repository, builds the graph, and returns a summary plus a viewer URL.

**When the agent should use it:** the user asks for an overview of a codebase, or any time understanding repo structure is necessary before further work.

**Schema:**

```json
{
  "name": "analyze_repo",
  "description": "Parse a code repository and return a structural summary plus a viewer URL. Use when you need an overview of a codebase or before any other Loom Lens query.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "Absolute path to the repo root."
      },
      "languages": {
        "type": "array",
        "items": {"enum": ["python", "typescript", "javascript", "rust"]},
        "description": "Languages to include. Default: all detected."
      },
      "include_external_calls": {
        "type": "boolean",
        "description": "Include calls into external libraries (tagged External). Default: false.",
        "default": false
      }
    },
    "required": ["path"]
  }
}
```

**Output:**
```json
{
  "graph_id": "abc123",
  "viewer_url": "http://localhost:7000/r/abc123",
  "summary": {
    "files": 142,
    "functions": 1830,
    "modules": 67,
    "languages": {"python": 87, "typescript": 55},
    "effect_distribution": {
      "Pure": 1240,
      "Net": 145,
      "IO": 88,
      "Mut": 220,
      "Throw": 410
    },
    "duplicate_count": 23,
    "cycle_count": 2
  }
}
```

---

### `query_graph`

Structured queries against an already-analyzed repo.

**When the agent should use it:** the user asks something specific (find effects, find callers, find duplicates), or the agent needs structural information about a portion of the codebase.

**Query kinds:**

| Kind | Purpose | Required parameters |
|------|---------|---------------------|
| `functions_with_effect` | Find functions inferred to have a given effect | `effect`, optional `min_confidence` |
| `duplicates_of` | Find functions with the same hash as a given function | `function` (file::name) |
| `callers_of` | Functions that call a given function | `function` |
| `callees_of` | Functions called by a given function | `function`, optional `transitive: bool` |
| `dependents_of_file` | Files that import or reference exports of a given file | `file` |
| `cycle_detection` | Module-level import cycles | (no extra params) |
| `unreachable_functions` | Functions with no callers, excluding entry points | optional `entry_points: [string]` |

**Schema:**
```json
{
  "name": "query_graph",
  "description": "Run a structured query against an analyzed repo.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "graph_id": {"type": "string"},
      "query": {
        "type": "object",
        "properties": {
          "kind": {"enum": ["functions_with_effect", "duplicates_of", "callers_of", "callees_of", "dependents_of_file", "cycle_detection", "unreachable_functions"]}
        },
        "required": ["kind"]
      }
    },
    "required": ["graph_id", "query"]
  }
}
```

**Output:**
```json
{
  "results": [
    {
      "node_id": "fn_142",
      "name": "fetch_user",
      "file": "src/api/users.py",
      "line": 42,
      "effects": [
        {"name": "Net", "confidence": "definite", "evidence": "fetch() at line 45"},
        {"name": "Throw", "confidence": "probable", "evidence": "raises HTTPError"}
      ],
      "hash": "blake3:7f3a..."
    }
  ],
  "viewer_url": "http://localhost:7000/r/abc123?filter=effect:Net"
}
```

---

### `get_function_context`

Returns source code, callers, and callees of a named function.

**When the agent should use it:** before editing or analyzing a specific function. Saves the agent from grep-ing the codebase.

**Schema:**
```json
{
  "name": "get_function_context",
  "description": "Return the source, callers, and callees of a specific function.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "graph_id": {"type": "string"},
      "function": {
        "type": "string",
        "description": "Function identifier in the form 'file::name', e.g., 'src/api/users.py::fetch_user'."
      }
    },
    "required": ["graph_id", "function"]
  }
}
```

**Output:** function info + arrays of callers/callees with their own info.

---

### `compare_hashes`

Compares two refs (commits, branches, working tree) and returns semantic-level changes.

**When the agent should use it:** the user asks "what changed?" — especially when `git diff` produces noise (whitespace, formatting, comments) but the agent needs to know what *behavior* changed.

**Schema:**
```json
{
  "name": "compare_hashes",
  "description": "Identify functions that changed semantically between two git refs. More precise than git diff for behavior changes.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {"type": "string"},
      "before": {"type": "string", "description": "Git ref or 'WORKING_TREE'"},
      "after": {"type": "string", "description": "Git ref or 'WORKING_TREE'"}
    },
    "required": ["path", "before", "after"]
  }
}
```

**Output:** four arrays — `changed`, `added`, `removed`, `renamed` — with hashes for each.

---

## HTTP-only endpoints (not exposed via MCP)

The viewer also exposes a few endpoints that are *not* MCP tools — they're for the browser:

- `GET /r/:graph_id` — viewer SPA
- `GET /api/graph/:graph_id` — graph data as JSON (consumed by the SPA)
- `GET /api/graph/:graph_id/source/:file` — raw source for a file (for the inline viewer)

These are documented for completeness but are not part of the agent's interface.

---

## Error handling

All tools return JSON-RPC errors with these codes:

| Code | Meaning |
|------|---------|
| `-32602` | Invalid params (bad path, missing graph_id, etc.) |
| `-32603` | Internal error |
| `1000` | Repo path not found |
| `1001` | Graph ID not found (call `analyze_repo` first) |
| `1002` | Language not supported |
| `1003` | Repo too large for current limits |
| `2000` | Function not found in graph |
| `2001` | Git ref not found |

Error responses include a `data` field with structured details for the agent to reason about.

---

## Versioning

The MCP API surface is versioned with the binary. v0.1.0 introduces this surface. Breaking changes will be batched into major version bumps; additions go in minor versions. The agent can detect the version by reading the MCP server's `server_info` response.
