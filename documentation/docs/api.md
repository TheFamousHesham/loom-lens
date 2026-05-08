# MCP Tool API

Loom Lens exposes four tools to Claude Code (and other MCP clients) over stdio JSON-RPC. This document is the implementation contract; design rationale is in `documentation/docs/decisions/0004-mcp-tool-api.md`.

> **Status:** Specified at Checkpoint 1; implemented at Checkpoint 2 (M1 minimal `analyze_repo`); fleshed out at Checkpoint 3 (M2 effects); refined at Checkpoint 4 (M3 hashes & polish). When implementation diverges from this document, fix the implementation, not the spec.

---

## Conventions

- **Transport.** stdio JSON-RPC 2.0, per the MCP specification. The server is launched by the client (`{"command": "loom-lens", "args": ["serve"]}`) and listens on stdin/stdout.
- **`graph_id`.** Returned by `analyze_repo`. Format: 12 lowercase hex chars (e.g., `7a3f9c1d2e8b`). Content-addressed: `blake3(canonical_repo_root_path || sorted_file_blake3s)`, truncated. Stable across re-analyses of the same repo at the same content; changes when any file changes. Held in an LRU cache of 8 graphs in the viewer process.
- **Function identifier (`function`).** `"<file_relative_to_repo_root>::<qualified_name>"`. For class methods: `"src/api/users.py::User.fetch"`. For free functions: `"src/api/users.py::fetch_user"`. Generated identifiers (lambdas, closures) follow `"src/file.py::<lambda@line:col>"`.
- **`viewer_url`.** Always `http://localhost:7000/r/<graph_id>` for the base graph; query strings filter the view (`?filter=effect:Net`, `?node=fn_142`).
- **Confidence.** Always one of `definite`, `probable`, `possible`. Used for both effect tags and result filters.
- **Errors.** JSON-RPC 2.0 error envelope with structured `data`. See "Error codes" below.

---

## Tools

### `analyze_repo`

Parse a repository, build the graph, and return a summary plus a viewer URL.

**Description (sent to the agent):**
> Parse a code repository and return a structural summary plus a viewer URL. **Use when** you need an overview of a codebase or before any other Loom Lens query — `query_graph`, `get_function_context`, and `compare_hashes` all need a `graph_id` from this tool. **Do not use** if you already have a fresh `graph_id` for the same repo at the same commit; reuse it. Re-analyzing is idempotent but costs ~1-30s on large repos.

**Input schema:**
```json
{
  "name": "analyze_repo",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {
        "type": "string",
        "description": "Absolute path to the repo root."
      },
      "languages": {
        "type": "array",
        "items": {"type": "string", "enum": ["python", "typescript", "javascript", "rust"]},
        "description": "Languages to include. Default: all detected."
      },
      "include_external_calls": {
        "type": "boolean",
        "default": false,
        "description": "Include calls into external libraries (tagged `External`). Default false; usually noise unless you specifically want library boundaries."
      },
      "max_files": {
        "type": "integer",
        "default": 10000,
        "description": "Hard cap on files parsed. Repos exceeding this return error 1003."
      }
    },
    "required": ["path"]
  }
}
```

**Output:**
```json
{
  "graph_id": "7a3f9c1d2e8b",
  "viewer_url": "http://localhost:7000/r/7a3f9c1d2e8b",
  "summary": {
    "files": 142,
    "functions": 1830,
    "modules": 67,
    "languages": {"python": 87, "typescript": 55},
    "effect_distribution": {
      "Pure": 1240, "Net": 145, "IO": 88, "Mut": 220,
      "Throw": 410, "Async": 95, "Random": 12, "Time": 31, "Foreign": 18
    },
    "duplicate_count": 23,
    "cycle_count": 2,
    "external_call_count": 0,
    "parse_errors": [
      {"file": "src/legacy/garbage.py", "line": 42, "message": "syntax error"}
    ]
  },
  "elapsed_ms": 8421
}
```

`parse_errors` is non-empty when individual files failed to parse; analysis continues with partial coverage and the rest of the summary reflects the parsed subset.

---

### `query_graph`

Run a structured query against an analyzed repo.

**Description (sent to the agent):**
> Run a structured query against an analyzed repo. **Use when** you need a list of functions matching a specific structural property (effect, hash equivalence, caller/callee membership, file dependency, cycle, reachability). **Do not use** for the source of a single function — use `get_function_context`. **Do not use** for cross-commit comparisons — use `compare_hashes`.

**Input schema:**
```json
{
  "name": "query_graph",
  "inputSchema": {
    "type": "object",
    "properties": {
      "graph_id": {"type": "string"},
      "query": {
        "type": "object",
        "oneOf": [
          {
            "properties": {
              "kind": {"const": "functions_with_effect"},
              "effect": {"enum": ["Net", "IO", "Mut", "Throw", "Async", "Random", "Time", "Foreign"]},
              "min_confidence": {"enum": ["definite", "probable", "possible"], "default": "possible"}
            },
            "required": ["kind", "effect"]
          },
          {
            "properties": {
              "kind": {"const": "duplicates_of"},
              "function": {"type": "string", "description": "file::name identifier"}
            },
            "required": ["kind", "function"]
          },
          {
            "properties": {
              "kind": {"const": "callers_of"},
              "function": {"type": "string"},
              "transitive": {"type": "boolean", "default": false}
            },
            "required": ["kind", "function"]
          },
          {
            "properties": {
              "kind": {"const": "callees_of"},
              "function": {"type": "string"},
              "transitive": {"type": "boolean", "default": false}
            },
            "required": ["kind", "function"]
          },
          {
            "properties": {
              "kind": {"const": "dependents_of_file"},
              "file": {"type": "string", "description": "Path relative to repo root"}
            },
            "required": ["kind", "file"]
          },
          {
            "properties": {
              "kind": {"const": "cycle_detection"}
            },
            "required": ["kind"]
          },
          {
            "properties": {
              "kind": {"const": "unreachable_functions"},
              "entry_points": {
                "type": "array",
                "items": {"type": "string"},
                "description": "function identifiers to treat as roots; default = main()/exported/test functions"
              }
            },
            "required": ["kind"]
          }
        ]
      },
      "limit": {"type": "integer", "default": 5000, "minimum": 1, "maximum": 50000},
      "offset": {"type": "integer", "default": 0, "minimum": 0}
    },
    "required": ["graph_id", "query"]
  }
}
```

**Output (uniform shape across query kinds):**
```json
{
  "results": [
    {
      "node_id": "fn_142",
      "name": "fetch_user",
      "qualified_name": "src/api/users.py::fetch_user",
      "file": "src/api/users.py",
      "line": 42,
      "language": "python",
      "signature": "def fetch_user(user_id: int) -> User",
      "effects": [
        {"name": "Net", "confidence": "definite", "evidence": "fetch() at line 45"},
        {"name": "Throw", "confidence": "probable", "evidence": "raises HTTPError"}
      ],
      "hash": "blake3:7f3a9c1d2e8b..."
    }
  ],
  "total_count": 145,
  "truncated": false,
  "viewer_url": "http://localhost:7000/r/7a3f9c1d2e8b?filter=effect:Net"
}
```

`truncated: true` indicates `total_count > limit`; client paginates by re-querying with `offset`. For `cycle_detection`, results are arrays of function identifiers (one cycle per array), not the standard node shape.

---

### `get_function_context`

Return source, callers, and callees of a named function.

**Description (sent to the agent):**
> Return the source code, signature, effects, hash, and immediate callers/callees of a specific function. **Use when** you need the source of a function before editing or analyzing it — replaces a grep + cat + manual call-site search. **Do not use** for repository-wide queries — use `query_graph`.

**Input schema:**
```json
{
  "name": "get_function_context",
  "inputSchema": {
    "type": "object",
    "properties": {
      "graph_id": {"type": "string"},
      "function": {
        "type": "string",
        "description": "Function identifier in the form 'file::name', e.g., 'src/api/users.py::fetch_user'."
      },
      "include_source": {"type": "boolean", "default": true},
      "max_callers": {"type": "integer", "default": 50},
      "max_callees": {"type": "integer", "default": 50}
    },
    "required": ["graph_id", "function"]
  }
}
```

**Output:**
```json
{
  "function": {
    "qualified_name": "src/api/users.py::fetch_user",
    "name": "fetch_user",
    "file": "src/api/users.py",
    "line_start": 42,
    "line_end": 58,
    "signature": "def fetch_user(user_id: int) -> User",
    "source": "def fetch_user(user_id: int) -> User:\n    ...",
    "effects": [
      {"name": "Net", "confidence": "definite", "evidence": "fetch() at line 45"}
    ],
    "hash": "blake3:7f3a9c1d2e8b..."
  },
  "callers": [
    {
      "qualified_name": "src/api/users.py::User.refresh",
      "name": "User.refresh",
      "file": "src/api/users.py",
      "line": 21,
      "call_sites": [{"line": 25, "column": 8}]
    }
  ],
  "callees": [
    {
      "qualified_name": "<external>::requests.get",
      "name": "requests.get",
      "file": "<external>",
      "line": null,
      "call_sites": [{"line": 45, "column": 12}],
      "is_external": true
    }
  ],
  "callers_truncated": false,
  "callees_truncated": false
}
```

If `include_source` is `false`, the `source` field is omitted (saves tokens for callers that already have the source).

---

### `compare_hashes`

Identify functions that changed semantically between two refs.

**Description (sent to the agent):**
> Identify functions that changed semantically between two git refs (commits, branches, tags). More precise than `git diff` for behavior changes — whitespace, formatting, comment, and import-only changes do not appear; renames are tracked by hash. **Use when** the user asks "what changed?" and wants behavior, not text. **Do not use** for understanding *why* something changed — read the diff itself.

**Input schema:**
```json
{
  "name": "compare_hashes",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": {"type": "string", "description": "Absolute path to the repo root."},
      "before": {
        "type": "string",
        "description": "Git ref, or the literal string \"WORKING_TREE\" for uncommitted state."
      },
      "after": {
        "type": "string",
        "description": "Git ref, or the literal string \"WORKING_TREE\" for uncommitted state."
      }
    },
    "required": ["path", "before", "after"]
  }
}
```

**Output:**
```json
{
  "before_ref": "HEAD~1",
  "before_commit": "a1b2c3d4...",
  "after_ref": "HEAD",
  "after_commit": "f5e6d7c8...",
  "changed": [
    {
      "qualified_name": "src/api/users.py::fetch_user",
      "before_hash": "blake3:7f3a...",
      "after_hash": "blake3:9d4a...",
      "kind": "modified"
    }
  ],
  "added": [
    {"qualified_name": "src/api/users.py::fetch_user_async", "hash": "blake3:abc1..."}
  ],
  "removed": [
    {"qualified_name": "src/api/users.py::deprecated_helper", "hash": "blake3:def2..."}
  ],
  "renamed": [
    {
      "before_qualified_name": "src/util/old_name.py::do_thing",
      "after_qualified_name": "src/util/new_name.py::do_thing",
      "hash": "blake3:c0c0..."
    }
  ],
  "viewer_url": "http://localhost:7000/r/7a3f9c1d2e8b/diff?before=a1b2c3d4&after=f5e6d7c8"
}
```

`renamed` requires identical hashes between the before and after sides; the rename is detected by hash equality plus location change. A semantic change *and* a rename appears in `changed` (with both qualified names recorded), not in `renamed`.

---

## HTTP-only endpoints (not exposed via MCP)

The viewer also exposes endpoints that are *not* MCP tools — they're for the browser:

| Method | Path | Purpose |
|--------|------|---------|
| GET | `/r/:graph_id` | Viewer SPA (HTML + bundled JS) |
| GET | `/api/graph/:graph_id` | Graph data as JSON (consumed by the SPA) |
| GET | `/api/graph/:graph_id/source/:file` | Raw source for a file (inline viewer) |
| GET | `/api/graph/:graph_id/diff?before=...&after=...` | `compare_hashes` JSON for the diff view |

These are documented for completeness but are not part of the agent's interface and are not stable across versions.

---

## Error codes

JSON-RPC errors follow this envelope:

```json
{
  "jsonrpc": "2.0",
  "id": 17,
  "error": {
    "code": 1001,
    "message": "graph_id not found",
    "data": {
      "graph_id": "7a3f9c1d2e8b",
      "hint": "Call analyze_repo first; graphs are cached LRU and may have been evicted."
    }
  }
}
```

Reserved code ranges (stable across versions):

| Range | Category |
|-------|----------|
| `-32700` to `-32603` | JSON-RPC standard errors (parse/method-not-found/internal) |
| `-32602` | Invalid params |
| `1000-1099` | Analysis errors (repo/path issues) |
| `2000-2099` | Graph-query errors (function/node not in graph) |
| `3000-3099` | Git/version errors (`compare_hashes`) |

Specific codes:

| Code | Meaning |
|------|---------|
| `-32602` | Invalid params (missing `path`, malformed `query`, etc.) |
| `-32603` | Internal error (parser crash, etc.) |
| `1000` | Repo path not found |
| `1001` | `graph_id` not found (call `analyze_repo` first) |
| `1002` | Language not supported |
| `1003` | Repo too large for current limits (`max_files` exceeded) |
| `1004` | Working directory not a git repo (only for tools that need git history) |
| `2000` | Function not found in graph |
| `2001` | File not found in graph |
| `2002` | Effect name not recognized |
| `3000` | Git ref not found |
| `3001` | Git ref ambiguous |

---

## Versioning

The MCP API surface is versioned with the binary. v0.1.0 introduces the surface defined in this document. Backward-compatible additions (new query kinds, new optional fields) go in minor versions; breaking changes (renaming a tool, removing a field, changing a field's type) require a major version bump and a superseding ADR. The agent can detect the version by reading the MCP server's `server_info` response.
