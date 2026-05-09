//! Build a `CodeGraph` from a collection of parsed files.
//!
//! Two passes:
//! 1. Per-file extraction (see `extract.rs`) yields nodes, contains-edges,
//!    and pending call/import records.
//! 2. Cross-file resolution turns pending records into `Calls` / `Imports`
//!    edges by name lookup against the global module/function index.
//!
//! Unresolved calls and imports are dropped at M1 (M2 will tag them
//! `External` per ADR 0002 §"External tag is its own dimension").

use crate::extract::{extract, PendingCall, PendingImport};
use crate::graph::{
    CodeGraph, Edge, EdgeKind, GraphId, Language, Node, NodeId, NodeKind, ParseErrorRecord,
    Summary,
};
use crate::parser::{parse_file, ParseError};
use crate::walk::{discover_files, DiscoveryOpts};
use indexmap::IndexMap;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use time::OffsetDateTime;

/// Top-level entry point: walk + parse + extract + build the graph.
pub fn analyze_repo(
    repo_root: &Path,
    opts: &DiscoveryOpts,
) -> Result<CodeGraph, crate::Error> {
    let started = Instant::now();
    let files = discover_files(repo_root, opts);

    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut pending_calls: Vec<PendingCall> = Vec::new();
    let mut pending_imports: Vec<PendingImport> = Vec::new();
    let mut parse_errors: Vec<ParseErrorRecord> = Vec::new();
    let mut next_id: u32 = 0;

    let mut file_blakes: Vec<String> = Vec::new();

    for rel in &files {
        let abs = repo_root.join(rel);
        match parse_file(&abs) {
            Ok(parsed) => {
                file_blakes.push(loom_lens_hashing_blake_or_inline(&parsed.source));
                let (extr, _file_id) = extract(&parsed, repo_root, &mut next_id);
                nodes.extend(extr.nodes);
                edges.extend(extr.edges);
                pending_calls.extend(extr.pending_calls);
                pending_imports.extend(extr.pending_imports);
            }
            Err(e) => {
                parse_errors.push(parse_error_record(&e));
            }
        }
    }

    // Resolve pending calls. Build a name → NodeId index per module first.
    let name_index = build_name_index(&nodes, &edges);
    for pc in pending_calls {
        if let Some(callee) = name_index.lookup_function(&pc.callee_name) {
            edges.push(Edge {
                from: pc.caller,
                to: callee,
                kind: EdgeKind::Calls,
                sites: vec![pc.site],
            });
        }
        // Unresolved: drop at M1; M2 attaches an External tag.
    }

    // Resolve pending imports: from_file → file owning module `module`.
    for pi in pending_imports {
        if let Some(target_file) = name_index.lookup_module_file(&pi.module) {
            edges.push(Edge {
                from: pi.from_file,
                to: target_file,
                kind: EdgeKind::Imports,
                sites: vec![pi.site],
            });
        }
    }

    let elapsed = started.elapsed();
    let summary = build_summary(&nodes, &parse_errors, elapsed.as_millis() as u64);

    let graph_id = GraphId::from_hash(&loom_lens_hashing_graph_id(
        repo_root.to_string_lossy().as_ref(),
        &file_blakes,
    ));

    Ok(CodeGraph {
        graph_id,
        repo_root: repo_root.to_path_buf(),
        nodes,
        edges,
        summary,
        generated_at: OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .unwrap_or_default(),
    })
}

/// Per-module name → NodeId index for resolving calls and imports.
struct NameIndex {
    /// All function/method nodes by bare name.
    functions: HashMap<String, NodeId>,
    /// Module name → file NodeId (so imports can attach to the importing file).
    modules: HashMap<String, NodeId>,
}

impl NameIndex {
    fn lookup_function(&self, name: &str) -> Option<NodeId> {
        self.functions.get(name).copied()
    }
    fn lookup_module_file(&self, module: &str) -> Option<NodeId> {
        self.modules.get(module).copied()
    }
}

fn build_name_index(nodes: &[Node], edges: &[Edge]) -> NameIndex {
    let mut functions: HashMap<String, NodeId> = HashMap::new();
    let mut modules: HashMap<String, NodeId> = HashMap::new();

    // First pass: function names.
    for n in nodes {
        if let NodeKind::Function { name, .. } = &n.kind {
            // First-wins: if multiple functions share a bare name, M1 picks the
            // first. M2 will resolve through imports for accuracy.
            functions.entry(name.clone()).or_insert(n.id);
        }
    }

    // Module → containing-file node id, by walking Contains edges.
    let mut module_nodes: HashMap<NodeId, String> = HashMap::new();
    let mut module_to_file: HashMap<NodeId, NodeId> = HashMap::new();
    for n in nodes {
        if let NodeKind::Module { name, file } = &n.kind {
            module_nodes.insert(n.id, name.clone());
            module_to_file.insert(n.id, *file);
        }
    }
    let _ = edges; // edges are not needed for the index at M1
    for (mod_id, name) in &module_nodes {
        if let Some(file_id) = module_to_file.get(mod_id) {
            modules.insert(name.clone(), *file_id);
        }
    }

    NameIndex { functions, modules }
}

fn build_summary(nodes: &[Node], parse_errors: &[ParseErrorRecord], elapsed_ms: u64) -> Summary {
    let mut s = Summary::default();
    let mut langs: IndexMap<String, u32> = IndexMap::new();
    for n in nodes {
        match &n.kind {
            NodeKind::File { language, .. } => {
                s.files += 1;
                let key = match language {
                    Language::Python => "python",
                    Language::Typescript => "typescript",
                    Language::Javascript => "javascript",
                    Language::Rust => "rust",
                };
                *langs.entry(key.to_string()).or_insert(0) += 1;
            }
            NodeKind::Function { .. } => s.functions += 1,
            NodeKind::Module { .. } => s.modules += 1,
            NodeKind::Type { .. } => {}
        }
    }
    s.languages = langs;
    s.parse_errors = parse_errors.to_vec();
    s.elapsed_ms = elapsed_ms;
    s
}

fn parse_error_record(err: &ParseError) -> ParseErrorRecord {
    match err {
        ParseError::Read { path, source } => ParseErrorRecord {
            file: path.clone(),
            line: 0,
            message: source.to_string(),
        },
        ParseError::Syntax { path, line } => ParseErrorRecord {
            file: path.clone(),
            line: *line,
            message: "syntax error".to_string(),
        },
        ParseError::LanguageLoad(_, msg) => ParseErrorRecord {
            file: PathBuf::new(),
            line: 0,
            message: msg.clone(),
        },
    }
}

// Inline hashing helpers so this crate doesn't depend on `loom-lens-hashing`,
// which would be a back-edge in the dependency graph (hashing depends on core).

fn loom_lens_hashing_blake_or_inline(bytes: &[u8]) -> String {
    let h = blake3::hash(bytes);
    hex::encode(h.as_bytes())
}

fn loom_lens_hashing_graph_id(repo_root: &str, file_hashes: &[String]) -> String {
    let mut h = blake3::Hasher::new();
    h.update(repo_root.as_bytes());
    h.update(b"\0");
    let mut sorted = file_hashes.to_vec();
    sorted.sort();
    for fh in &sorted {
        h.update(fh.as_bytes());
        h.update(b"\0");
    }
    let full = h.finalize();
    hex::encode(&full.as_bytes()[..6])
}
