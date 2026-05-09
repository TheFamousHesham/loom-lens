//! Build a `CodeGraph` from a collection of parsed files.
//!
//! Three passes:
//!
//! 1. Per-file extraction (see `extract.rs`) emits nodes, contains-edges,
//!    pending call records, and pending import records.
//! 2. Canonical Python module-name assignment by walking `__init__.py`
//!    boundaries from each file's directory upward. Module nodes get their
//!    `name` field rewritten to the canonical Python import path.
//! 3. Cross-file resolution: pending records become `Calls`/`Imports` edges
//!    by lookup against the canonical module-name index, the per-module
//!    function index, and a per-file imports table built from successfully
//!    resolved imports.
//!
//! Unresolved calls and imports are dropped at M1 polish (M2 will tag them
//! with the `External` provenance per ADR 0002).

use crate::extract::{extract, CallKind, PendingCall, PendingImport};
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

/// Top-level entry point: walk + parse + extract + canonicalise + resolve.
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
                file_blakes.push(blake3_hex_of(&parsed.source));
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

    // Pass 2: canonical module names for every Python file.
    let canonical_by_file = discover_python_packages(repo_root, &files);
    rewrite_module_names(&mut nodes, &canonical_by_file);

    // Pass 3: resolution.
    let index = NameIndex::build(&nodes, &edges);

    let (imports_resolved, total_imports, per_file_imports) =
        resolve_imports(&pending_imports, &index, &mut edges);

    let (calls_resolved, total_calls) =
        resolve_calls(&pending_calls, &index, &per_file_imports, &mut edges);

    let elapsed = started.elapsed();
    let mut summary = build_summary(&nodes, &parse_errors, elapsed.as_millis() as u64);
    summary.calls_resolved = calls_resolved;
    summary.calls_total = total_calls;
    summary.imports_resolved = imports_resolved;
    summary.imports_total = total_imports;

    let graph_id = GraphId::from_hash(&graph_id_hex(
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

// ───────────────────────────────────────────────────────────────
// Canonical Python module names
// ───────────────────────────────────────────────────────────────

/// Compute the canonical Python import path for each `.py` file by walking
/// the `__init__.py` chain from the file's directory upward. A directory is
/// part of a package iff it contains `__init__.py`. The walk stops at the
/// first directory that is NOT a package; everything below joins with `.` to
/// form the canonical name. Files outside any package fall back to the
/// path-as-dots form (e.g. `setup.py` → `setup`).
pub fn discover_python_packages(
    repo_root: &Path,
    files: &[PathBuf],
) -> HashMap<PathBuf, String> {
    let mut out = HashMap::new();
    for rel in files {
        let ext = rel.extension().and_then(|s| s.to_str());
        if ext != Some("py") && ext != Some("pyi") {
            continue;
        }
        let mut parts: Vec<String> = Vec::new();
        let mut current: Option<&Path> = rel.parent();
        while let Some(p) = current {
            if p.as_os_str().is_empty() {
                break;
            }
            let init_py = repo_root.join(p).join("__init__.py");
            if init_py.exists() {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    parts.push(name.to_string());
                }
                current = p.parent();
            } else {
                break;
            }
        }
        parts.reverse();
        let stem = rel
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_string();
        let canonical = if parts.is_empty() {
            crate::extract::python_module_name(rel)
        } else if stem == "__init__" {
            parts.join(".")
        } else {
            let mut full = parts;
            full.push(stem);
            full.join(".")
        };
        out.insert(rel.clone(), canonical);
    }
    out
}

fn rewrite_module_names(nodes: &mut [Node], canonical: &HashMap<PathBuf, String>) {
    let file_path_by_id: HashMap<NodeId, PathBuf> = nodes
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::File { path, .. } => Some((n.id, path.clone())),
            _ => None,
        })
        .collect();
    for n in nodes {
        if let NodeKind::Module { name, file } = &mut n.kind {
            if let Some(path) = file_path_by_id.get(file) {
                if let Some(canon) = canonical.get(path) {
                    *name = canon.clone();
                }
            }
        }
    }
}

/// Compute the canonical name of the module that a `from X import Y`
/// statement targets, given the file's own canonical name, whether the file
/// is an `__init__.py` (its canonical IS the package), and the import's
/// `level` of leading dots and `module` text.
fn compute_target_module(
    file_canonical: &str,
    file_is_init: bool,
    level: u32,
    module: &str,
) -> String {
    if level == 0 {
        return module.to_string();
    }
    let mut parts: Vec<&str> = file_canonical.split('.').collect();
    // For non-__init__ files the canonical is `<package>.<file>`; drop the
    // file component to leave the file's containing package. For __init__.py
    // the canonical IS the package, so don't drop.
    if !file_is_init {
        parts.pop();
    }
    for _ in 0..level.saturating_sub(1) {
        parts.pop();
    }
    let base = parts.join(".");
    if module.is_empty() {
        base
    } else if base.is_empty() {
        module.to_string()
    } else {
        format!("{base}.{module}")
    }
}

// ───────────────────────────────────────────────────────────────
// Index
// ───────────────────────────────────────────────────────────────

struct NameIndex {
    /// canonical_module_name → file NodeId
    module_to_file: HashMap<String, NodeId>,
    /// file NodeId → canonical_module_name
    file_to_module: HashMap<NodeId, String>,
    /// file NodeId → true iff the file's basename is `__init__.py` (or `.pyi`).
    file_is_init: HashMap<NodeId, bool>,
    /// File NodeId → its Module NodeId.
    file_to_module_node: HashMap<NodeId, NodeId>,
    /// Function NodeId → File NodeId (so the call resolver can find context).
    fn_to_file: HashMap<NodeId, NodeId>,
    /// Per-module top-level functions: module NodeId → name → fn NodeId.
    fns_in_module: HashMap<NodeId, HashMap<String, NodeId>>,
    /// Per-class methods: class NodeId → method name → fn NodeId.
    methods_in_class: HashMap<NodeId, HashMap<String, NodeId>>,
}

impl NameIndex {
    fn build(nodes: &[Node], edges: &[Edge]) -> Self {
        let mut module_to_file: HashMap<String, NodeId> = HashMap::new();
        let mut file_to_module: HashMap<NodeId, String> = HashMap::new();
        let mut file_is_init: HashMap<NodeId, bool> = HashMap::new();
        let mut file_to_module_node: HashMap<NodeId, NodeId> = HashMap::new();
        let mut fn_to_file: HashMap<NodeId, NodeId> = HashMap::new();
        let mut fns_in_module: HashMap<NodeId, HashMap<String, NodeId>> = HashMap::new();
        let mut methods_in_class: HashMap<NodeId, HashMap<String, NodeId>> = HashMap::new();

        for n in nodes {
            if let NodeKind::File { path, .. } = &n.kind {
                let is_init = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(|s| s == "__init__")
                    .unwrap_or(false);
                file_is_init.insert(n.id, is_init);
            }
        }
        for n in nodes {
            if let NodeKind::Module { name, file } = &n.kind {
                module_to_file.insert(name.clone(), *file);
                file_to_module.insert(*file, name.clone());
                file_to_module_node.insert(*file, n.id);
            }
        }

        let by_id: HashMap<NodeId, &Node> = nodes.iter().map(|n| (n.id, n)).collect();
        let mut module_to_file_node: HashMap<NodeId, NodeId> = HashMap::new();
        for e in edges {
            if e.kind != EdgeKind::Contains {
                continue;
            }
            let Some(parent) = by_id.get(&e.from) else {
                continue;
            };
            let Some(child) = by_id.get(&e.to) else {
                continue;
            };
            match (&parent.kind, &child.kind) {
                (NodeKind::File { .. }, NodeKind::Module { .. }) => {
                    module_to_file_node.insert(child.id, parent.id);
                }
                (NodeKind::Module { .. }, NodeKind::Function { name, .. }) => {
                    if let Some(file_id) = module_to_file_node.get(&parent.id).copied() {
                        fn_to_file.insert(child.id, file_id);
                    }
                    // Only top-level functions (not class methods) go in the
                    // module's function index. We tell them apart by the
                    // qualified_name format — class methods carry "<file>::<Class>.<method>".
                    if let NodeKind::Function {
                        qualified_name, ..
                    } = &child.kind
                    {
                        let is_method = qualified_name
                            .split_once("::")
                            .map(|(_, after)| after.contains('.'))
                            .unwrap_or(false);
                        if !is_method {
                            fns_in_module
                                .entry(parent.id)
                                .or_default()
                                .insert(name.clone(), child.id);
                        }
                    }
                }
                _ => {}
            }
        }

        // Methods-by-class: pair functions whose qualified_name carries a
        // "<Class>." infix with the matching Type node.
        let types_by_qualified: HashMap<&str, NodeId> = nodes
            .iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Type { qualified_name, .. } => Some((qualified_name.as_str(), n.id)),
                _ => None,
            })
            .collect();
        for n in nodes {
            if let NodeKind::Function { qualified_name, .. } = &n.kind {
                if let Some((prefix, method)) = qualified_name.rsplit_once('.') {
                    if prefix.contains("::") {
                        if let Some(class_id) = types_by_qualified.get(prefix) {
                            methods_in_class
                                .entry(*class_id)
                                .or_default()
                                .insert(method.to_string(), n.id);
                        }
                    }
                }
            }
        }

        Self {
            module_to_file,
            file_to_module,
            file_is_init,
            file_to_module_node,
            fn_to_file,
            fns_in_module,
            methods_in_class,
        }
    }
}

// ───────────────────────────────────────────────────────────────
// Resolution: imports
// ───────────────────────────────────────────────────────────────

type PerFileImports = HashMap<NodeId, HashMap<String, NodeId>>;

fn resolve_imports(
    pending: &[PendingImport],
    index: &NameIndex,
    edges: &mut Vec<Edge>,
) -> (u32, u32, PerFileImports) {
    let mut imports_resolved: u32 = 0;
    let total = pending.len() as u32;
    let mut per_file_imports: PerFileImports = HashMap::new();
    let mut imports_acc: HashMap<(NodeId, NodeId), Vec<crate::graph::Span>> = HashMap::new();

    for pi in pending {
        let Some(file_canonical) = index.file_to_module.get(&pi.from_file) else {
            continue;
        };
        let is_init = index.file_is_init.get(&pi.from_file).copied().unwrap_or(false);
        let target_module = compute_target_module(file_canonical, is_init, pi.level, &pi.module);

        let target_file_id = if let Some(ref imported_name) = pi.name {
            // `from X import Y`: Y might be a submodule (its own file) or a
            // symbol within X. Try the submodule path first.
            let candidate = if target_module.is_empty() {
                imported_name.clone()
            } else {
                format!("{target_module}.{imported_name}")
            };
            index
                .module_to_file
                .get(&candidate)
                .copied()
                .or_else(|| index.module_to_file.get(&target_module).copied())
        } else {
            // `import X` / `import X.Y` — wire to whatever file owns the name.
            index.module_to_file.get(&target_module).copied()
        };

        let Some(target_file) = target_file_id else {
            continue;
        };
        if target_file == pi.from_file {
            continue;
        }
        imports_resolved += 1;
        imports_acc
            .entry((pi.from_file, target_file))
            .or_default()
            .push(pi.site);

        // Per-file imports table — keyed by the LOCALLY-bound name (alias if
        // present, else the imported name, else the leftmost module segment).
        let local_alias = pi.alias.clone().or_else(|| pi.name.clone()).unwrap_or_else(|| {
            target_module
                .split('.')
                .next()
                .unwrap_or("")
                .to_string()
        });
        if !local_alias.is_empty() {
            per_file_imports
                .entry(pi.from_file)
                .or_default()
                .insert(local_alias, target_file);
        }
    }

    for ((from, to), sites) in imports_acc {
        edges.push(Edge {
            from,
            to,
            kind: EdgeKind::Imports,
            sites,
        });
    }

    (imports_resolved, total, per_file_imports)
}

// ───────────────────────────────────────────────────────────────
// Resolution: calls
// ───────────────────────────────────────────────────────────────

fn resolve_calls(
    pending: &[PendingCall],
    index: &NameIndex,
    per_file_imports: &PerFileImports,
    edges: &mut Vec<Edge>,
) -> (u32, u32) {
    let mut resolved: u32 = 0;
    let total = pending.len() as u32;
    for pc in pending {
        if let Some(callee) = resolve_one_call(pc, index, per_file_imports) {
            resolved += 1;
            edges.push(Edge {
                from: pc.caller,
                to: callee,
                kind: EdgeKind::Calls,
                sites: vec![pc.site],
            });
        }
    }
    (resolved, total)
}

fn resolve_one_call(
    pc: &PendingCall,
    index: &NameIndex,
    per_file_imports: &PerFileImports,
) -> Option<NodeId> {
    let caller_file = index.fn_to_file.get(&pc.caller).copied()?;
    match &pc.kind {
        CallKind::Bare { name } => {
            // 1) Same module top-level.
            if let Some(module_id) = index.file_to_module_node.get(&caller_file).copied() {
                if let Some(id) = index
                    .fns_in_module
                    .get(&module_id)
                    .and_then(|m| m.get(name).copied())
                {
                    return Some(id);
                }
            }
            // 2) Imports table — `from X import foo` makes a bare `foo()`
            //    point at function `foo` in module X.
            let imports = per_file_imports.get(&caller_file)?;
            let target_file = imports.get(name)?;
            let module_id = index.file_to_module_node.get(target_file).copied()?;
            index
                .fns_in_module
                .get(&module_id)
                .and_then(|m| m.get(name).copied())
        }
        CallKind::SelfMethod {
            method,
            enclosing_class,
        } => index
            .methods_in_class
            .get(enclosing_class)
            .and_then(|m| m.get(method).copied()),
    }
}

// ───────────────────────────────────────────────────────────────
// Misc
// ───────────────────────────────────────────────────────────────

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
            NodeKind::Type { .. } => s.types += 1,
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

fn blake3_hex_of(bytes: &[u8]) -> String {
    let h = blake3::hash(bytes);
    hex::encode(h.as_bytes())
}

fn graph_id_hex(repo_root: &str, file_hashes: &[String]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_module_relative_from_module_file() {
        // `from .X import Y` inside `requests/sessions.py` → `requests.X`
        assert_eq!(
            compute_target_module("requests.sessions", false, 1, "X"),
            "requests.X"
        );
        // `from . import X` inside `requests/sessions.py` → `requests`
        assert_eq!(
            compute_target_module("requests.sessions", false, 1, ""),
            "requests"
        );
        // `from ..X import Y` inside `pkg/subpkg/mod.py` → `pkg.X`
        assert_eq!(
            compute_target_module("pkg.subpkg.mod", false, 2, "X"),
            "pkg.X"
        );
        // Absolute import is identity regardless of init-ness.
        assert_eq!(
            compute_target_module("requests.sessions", false, 0, "json"),
            "json"
        );
    }

    #[test]
    fn target_module_relative_from_init() {
        // `from .X import Y` inside `requests/__init__.py` (canonical "requests") → `requests.X`
        assert_eq!(
            compute_target_module("requests", true, 1, "X"),
            "requests.X"
        );
        // `from . import X` inside `requests/__init__.py` → `requests`
        assert_eq!(
            compute_target_module("requests", true, 1, ""),
            "requests"
        );
        // `from .. import X` inside `pkg/sub/__init__.py` (canonical "pkg.sub") → `pkg`
        assert_eq!(compute_target_module("pkg.sub", true, 2, ""), "pkg");
    }
}
