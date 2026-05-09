//! Tool descriptor table sent in `tools/list`.
//!
//! Schemas mirror `documentation/docs/api.md`; updates land here and
//! propagate via the `tools/list` response.

use serde_json::{json, Value};

/// The four tools as JSON Schema descriptors.
#[must_use]
pub fn tool_descriptors() -> Vec<Value> {
    vec![analyze_repo(), query_graph(), get_function_context(), compare_hashes()]
}

fn analyze_repo() -> Value {
    json!({
        "name": "analyze_repo",
        "description": "Parse a code repository and return a structural summary plus a viewer URL. Use when you need an overview of a codebase or before any other Loom Lens query — query_graph, get_function_context, and compare_hashes all need a graph_id from this tool. Do not use if you already have a fresh graph_id for the same repo at the same commit; reuse it.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": { "type": "string", "description": "Absolute path to the repo root." },
                "languages": {
                    "type": "array",
                    "items": { "type": "string", "enum": ["python", "typescript", "javascript", "rust"] },
                    "description": "Languages to include. Default: all detected."
                },
                "include_external_calls": { "type": "boolean", "default": false },
                "max_files": { "type": "integer", "default": 10000 }
            },
            "required": ["path"]
        }
    })
}

fn query_graph() -> Value {
    json!({
        "name": "query_graph",
        "description": "Run a structured query against an analyzed repo. Use for lists of functions matching a structural property (effect, hash equivalence, caller/callee membership, file dependency, cycle, reachability). Do not use for the source of a single function — use get_function_context. Do not use for cross-commit comparisons — use compare_hashes.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "graph_id": { "type": "string" },
                "query": { "type": "object" },
                "limit": { "type": "integer", "default": 5000 },
                "offset": { "type": "integer", "default": 0 }
            },
            "required": ["graph_id", "query"]
        }
    })
}

fn get_function_context() -> Value {
    json!({
        "name": "get_function_context",
        "description": "Return the source code, signature, effects, hash, and immediate callers/callees of a specific function. Use before editing or analyzing a specific function. Do not use for repository-wide queries — use query_graph.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "graph_id": { "type": "string" },
                "function": { "type": "string", "description": "file::name identifier" },
                "include_source": { "type": "boolean", "default": true },
                "max_callers": { "type": "integer", "default": 50 },
                "max_callees": { "type": "integer", "default": 50 }
            },
            "required": ["graph_id", "function"]
        }
    })
}

fn compare_hashes() -> Value {
    json!({
        "name": "compare_hashes",
        "description": "Identify functions that changed semantically between two git refs. More precise than git diff for behavior changes — whitespace, formatting, comment, and import-only changes do not appear. Use when the user asks 'what changed?' and wants behavior, not text. Do not use for understanding why something changed — read the diff itself.",
        "inputSchema": {
            "type": "object",
            "properties": {
                "path": { "type": "string" },
                "before": { "type": "string" },
                "after": { "type": "string" }
            },
            "required": ["path", "before", "after"]
        }
    })
}
