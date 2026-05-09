//! Code-graph data model. See `documentation/ARCHITECTURE.md` §4.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use time::OffsetDateTime;

/// Content-addressed graph identifier (12 hex chars; see ADR 0004 refinements).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct GraphId(pub String);

impl GraphId {
    /// Build from a content hash, truncating to 12 hex chars.
    #[must_use]
    pub fn from_hash(full: &str) -> Self {
        Self(full.chars().take(12).collect())
    }
}

impl std::fmt::Display for GraphId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Stable per-graph node handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct NodeId(pub u32);

/// Source language tag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    /// Python.
    Python,
    /// TypeScript.
    Typescript,
    /// JavaScript.
    Javascript,
    /// Rust.
    Rust,
}

impl Language {
    /// Map a file extension to a language tag, if recognised.
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "py" | "pyi" => Some(Self::Python),
            "ts" | "tsx" => Some(Self::Typescript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Self::Javascript),
            "rs" => Some(Self::Rust),
            _ => None,
        }
    }
}

/// Source range — byte offsets and 1-based line/column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// Inclusive byte start.
    pub byte_start: u32,
    /// Exclusive byte end.
    pub byte_end: u32,
    /// 1-based start line.
    pub line_start: u32,
    /// 1-based end line.
    pub line_end: u32,
    /// 1-based start column.
    pub col_start: u32,
    /// 1-based end column.
    pub col_end: u32,
}

/// Concrete node kinds; see ARCHITECTURE.md §4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NodeKind {
    /// A source file.
    File {
        /// Path relative to the repo root.
        path: PathBuf,
        /// Detected language.
        language: Language,
        /// Line count (after parsing).
        lines: u32,
    },
    /// A module (Python module, TS namespace, Rust mod).
    Module {
        /// Dotted/qualified name (`pkg.subpkg.mod`).
        name: String,
        /// File this module lives in.
        file: NodeId,
    },
    /// A function or method.
    Function {
        /// Bare name (without enclosing class/module).
        name: String,
        /// Qualified `file::path::name` identifier (see api.md conventions).
        qualified_name: String,
        /// Source signature line (best-effort).
        signature: String,
    },
    /// A type/class.
    Type {
        /// Bare name.
        name: String,
        /// Qualified identifier.
        qualified_name: String,
    },
}

/// A graph node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Node {
    /// Stable handle.
    pub id: NodeId,
    /// Kind-specific payload.
    #[serde(flatten)]
    pub kind: NodeKind,
    /// Source span (file-only nodes carry the whole-file span; modules carry their declaration; etc.).
    pub span: Span,
}

/// Edge variants; see ARCHITECTURE.md §4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    /// File → module, module → function/type, etc.
    Contains,
    /// Function → function (call site).
    Calls,
    /// Module → module (import).
    Imports,
    /// Function → type (parameter / return / member access).
    References,
}

/// An edge. Call sites and import lines are recorded on the edge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edge {
    /// Source node.
    pub from: NodeId,
    /// Destination node.
    pub to: NodeId,
    /// Edge kind.
    pub kind: EdgeKind,
    /// Source spans where the relationship is realised (call sites, import statements).
    pub sites: Vec<Span>,
}

/// Aggregate counts surfaced by the `analyze_repo` MCP tool.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// Total files parsed.
    pub files: u32,
    /// Total functions discovered.
    pub functions: u32,
    /// Total modules discovered.
    pub modules: u32,
    /// Per-language file counts.
    pub languages: IndexMap<String, u32>,
    /// Wallclock spent in `analyze_repo`.
    pub elapsed_ms: u64,
    /// Files that failed to parse, paired with the parser error.
    pub parse_errors: Vec<ParseErrorRecord>,
}

/// A parse-error record carried in `Summary`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseErrorRecord {
    /// File that failed.
    pub file: PathBuf,
    /// Best-effort line of failure (0 if unknown).
    pub line: u32,
    /// Human-readable message.
    pub message: String,
}

/// The whole repo as a graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    /// Content-addressed identifier.
    pub graph_id: GraphId,
    /// Repo root the graph was built from.
    pub repo_root: PathBuf,
    /// All nodes, addressable by `NodeId`.
    pub nodes: Vec<Node>,
    /// All edges.
    pub edges: Vec<Edge>,
    /// Summary stats.
    pub summary: Summary,
    /// Generation timestamp (UTC, ISO-8601).
    pub generated_at: String,
}

impl CodeGraph {
    /// Empty graph for a given repo root and graph id.
    #[must_use]
    pub fn empty(graph_id: GraphId, repo_root: PathBuf) -> Self {
        Self {
            graph_id,
            repo_root,
            nodes: Vec::new(),
            edges: Vec::new(),
            summary: Summary::default(),
            generated_at: OffsetDateTime::now_utc()
                .format(&time::format_description::well_known::Rfc3339)
                .unwrap_or_default(),
        }
    }

    /// Find a node by id.
    #[must_use]
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.0 as usize)
    }
}
