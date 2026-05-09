//! AST → graph extraction for a single parsed file.
//!
//! Produces:
//! - One `File` node and one `Module` node per file.
//! - One `Function` node per top-level `def`/`async def` and per class method.
//! - One `Type` node per top-level `class`.
//! - `Contains` edges from file → module → function/type.
//! - `PendingCall` records (caller_node_id, callee_name, span) — resolved to
//!   `Calls` edges in the second pass at graph-build time.
//! - `PendingImport` records — resolved to `Imports` edges across modules.

use crate::graph::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
use crate::parser::ParsedFile;
use std::path::Path;
use tree_sitter::Node as TsNode;

/// A call site whose callee is recorded by name; resolution to a `NodeId`
/// happens after all files are parsed and a global name index is built.
#[derive(Debug, Clone)]
pub struct PendingCall {
    /// Caller function node.
    pub caller: NodeId,
    /// Bare callee name (e.g., `fetch_user` for `fetch_user(...)`).
    /// Attribute calls record the rightmost segment (e.g., `client.get(...)`
    /// becomes `get`); cross-attribute resolution is M2.
    pub callee_name: String,
    /// Source span of the call expression.
    pub site: Span,
}

/// A module-level import statement; `from X import Y` produces one record per Y,
/// with `module = X` and `name = Y`.
#[derive(Debug, Clone)]
pub struct PendingImport {
    /// Importing file's node id.
    pub from_file: NodeId,
    /// Dotted module name (e.g., `requests.exceptions`).
    pub module: String,
    /// Specific name imported (e.g., `Session`); `None` for `import x`.
    pub name: Option<String>,
    /// Span of the import statement.
    pub site: Span,
}

/// Per-file extraction output. The caller threads a `next_id` counter so
/// `NodeId`s stay globally unique.
#[derive(Debug, Default)]
pub struct Extraction {
    /// New nodes produced by this file.
    pub nodes: Vec<Node>,
    /// Containment edges produced by this file.
    pub edges: Vec<Edge>,
    /// Unresolved call sites; resolved during graph build.
    pub pending_calls: Vec<PendingCall>,
    /// Unresolved imports; resolved during graph build.
    pub pending_imports: Vec<PendingImport>,
}

/// Run extraction over `parsed`. Returns the extraction plus the assigned
/// `NodeId` of the file node (useful when the caller wants to track files).
pub fn extract(parsed: &ParsedFile, repo_root: &Path, next_id: &mut u32) -> (Extraction, NodeId) {
    let mut out = Extraction::default();
    let rel = parsed
        .path
        .strip_prefix(repo_root)
        .unwrap_or(&parsed.path)
        .to_path_buf();
    let lines = parsed.source.iter().filter(|&&b| b == b'\n').count() as u32 + 1;

    let file_id = NodeId(*next_id);
    *next_id += 1;
    out.nodes.push(Node {
        id: file_id,
        kind: NodeKind::File {
            path: rel.clone(),
            language: parsed.language,
            lines,
        },
        span: span_of(parsed.tree.root_node()),
    });

    if parsed.language == Language::Python {
        let module_name = python_module_name(&rel);
        let module_id = NodeId(*next_id);
        *next_id += 1;
        out.nodes.push(Node {
            id: module_id,
            kind: NodeKind::Module {
                name: module_name,
                file: file_id,
            },
            span: span_of(parsed.tree.root_node()),
        });
        out.edges.push(Edge {
            from: file_id,
            to: module_id,
            kind: EdgeKind::Contains,
            sites: vec![],
        });
        let mut ctx = PyCtx {
            source: &parsed.source,
            file_path: &rel,
            file_id,
            module_id,
            next_id,
            current_function: None,
            current_class: None,
        };
        visit_python(parsed.tree.root_node(), &mut ctx, &mut out);
    }

    (out, file_id)
}

struct PyCtx<'a> {
    source: &'a [u8],
    file_path: &'a Path,
    file_id: NodeId,
    module_id: NodeId,
    next_id: &'a mut u32,
    current_function: Option<NodeId>,
    current_class: Option<String>,
}

fn visit_python(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    match node.kind() {
        "module" => visit_children(node, ctx, out),
        "function_definition" | "async_function_definition" => {
            handle_function(node, ctx, out);
        }
        "class_definition" => {
            handle_class(node, ctx, out);
        }
        "call" => {
            handle_call(node, ctx, out);
            visit_children(node, ctx, out);
        }
        "import_statement" => {
            handle_import_statement(node, ctx, out);
        }
        "import_from_statement" => {
            handle_import_from(node, ctx, out);
        }
        // The `decorated_definition` node wraps function_definition / class_definition.
        "decorated_definition" => visit_children(node, ctx, out),
        _ => visit_children(node, ctx, out),
    }
}

fn visit_children(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        visit_python(child, ctx, out);
    }
}

fn handle_function(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    let name = node_text(name_node, ctx.source).to_string();
    let qualified = match &ctx.current_class {
        Some(cls) => format!("{}::{}.{}", ctx.file_path.display(), cls, name),
        None => format!("{}::{}", ctx.file_path.display(), name),
    };
    let signature = first_line(node_text(node, ctx.source));

    let fn_id = NodeId(*ctx.next_id);
    *ctx.next_id += 1;
    out.nodes.push(Node {
        id: fn_id,
        kind: NodeKind::Function {
            name,
            qualified_name: qualified,
            signature,
        },
        span: span_of(node),
    });
    out.edges.push(Edge {
        from: ctx.module_id,
        to: fn_id,
        kind: EdgeKind::Contains,
        sites: vec![],
    });

    // Body: traverse with this function as the current caller scope.
    let saved_fn = ctx.current_function;
    ctx.current_function = Some(fn_id);
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            visit_python(child, ctx, out);
        }
    }
    ctx.current_function = saved_fn;
}

fn handle_class(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    let Some(name_node) = node.child_by_field_name("name") else {
        return;
    };
    let class_name = node_text(name_node, ctx.source).to_string();
    let qualified = format!("{}::{}", ctx.file_path.display(), class_name);

    let class_id = NodeId(*ctx.next_id);
    *ctx.next_id += 1;
    out.nodes.push(Node {
        id: class_id,
        kind: NodeKind::Type {
            name: class_name.clone(),
            qualified_name: qualified,
        },
        span: span_of(node),
    });
    out.edges.push(Edge {
        from: ctx.module_id,
        to: class_id,
        kind: EdgeKind::Contains,
        sites: vec![],
    });

    let saved_class = ctx.current_class.take();
    ctx.current_class = Some(class_name);
    if let Some(body) = node.child_by_field_name("body") {
        let mut cursor = body.walk();
        for child in body.children(&mut cursor) {
            visit_python(child, ctx, out);
        }
    }
    ctx.current_class = saved_class;
}

fn handle_call(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    let Some(caller) = ctx.current_function else {
        return; // module-level call: no caller scope to attach to.
    };
    let Some(fn_expr) = node.child_by_field_name("function") else {
        return;
    };
    let name = match fn_expr.kind() {
        "identifier" => node_text(fn_expr, ctx.source).to_string(),
        "attribute" => fn_expr
            .child_by_field_name("attribute")
            .map(|n| node_text(n, ctx.source).to_string())
            .unwrap_or_default(),
        _ => return,
    };
    if name.is_empty() {
        return;
    }
    out.pending_calls.push(PendingCall {
        caller,
        callee_name: name,
        site: span_of(node),
    });
}

fn handle_import_statement(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    // `import a.b.c` — child_by_field_name("name") returns the dotted_name.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "dotted_name" | "aliased_import") {
            let module = node_text(child, ctx.source).to_string();
            // Strip ' as alias' if present.
            let module = module
                .split_whitespace()
                .next()
                .unwrap_or(&module)
                .to_string();
            out.pending_imports.push(PendingImport {
                from_file: ctx.file_id,
                module,
                name: None,
                site: span_of(node),
            });
        }
    }
}

fn handle_import_from(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    let module = node
        .child_by_field_name("module_name")
        .map(|n| node_text(n, ctx.source).to_string())
        .unwrap_or_default();
    if module.is_empty() {
        return;
    }
    // Each imported name produces one record.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if matches!(child.kind(), "dotted_name") && child.id() != node.id() {
            let txt = node_text(child, ctx.source).to_string();
            if txt == module {
                continue; // skip the module_name itself
            }
            out.pending_imports.push(PendingImport {
                from_file: ctx.file_id,
                module: module.clone(),
                name: Some(txt),
                site: span_of(node),
            });
        } else if child.kind() == "aliased_import" {
            if let Some(name_n) = child.child_by_field_name("name") {
                let txt = node_text(name_n, ctx.source).to_string();
                out.pending_imports.push(PendingImport {
                    from_file: ctx.file_id,
                    module: module.clone(),
                    name: Some(txt),
                    site: span_of(node),
                });
            }
        }
    }
}

fn span_of(node: TsNode<'_>) -> Span {
    let s = node.start_position();
    let e = node.end_position();
    Span {
        byte_start: node.start_byte() as u32,
        byte_end: node.end_byte() as u32,
        line_start: s.row as u32 + 1,
        line_end: e.row as u32 + 1,
        col_start: s.column as u32 + 1,
        col_end: e.column as u32 + 1,
    }
}

fn node_text<'a>(node: TsNode<'_>, source: &'a [u8]) -> &'a str {
    std::str::from_utf8(&source[node.byte_range()]).unwrap_or("")
}

fn first_line(text: &str) -> String {
    text.lines().next().unwrap_or("").trim_end().to_string()
}

fn python_module_name(rel: &Path) -> String {
    let mut parts: Vec<String> = Vec::new();
    for c in rel.components() {
        let s = c.as_os_str().to_string_lossy();
        let stripped = s
            .strip_suffix(".py")
            .or_else(|| s.strip_suffix(".pyi"))
            .unwrap_or(s.as_ref())
            .to_string();
        if stripped == "__init__" {
            continue;
        }
        parts.push(stripped);
    }
    let joined = parts.join(".");
    if joined.is_empty() {
        rel.display().to_string()
    } else {
        joined
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn module_name_canonical() {
        assert_eq!(
            python_module_name(&PathBuf::from("src/api/users.py")),
            "src.api.users"
        );
        assert_eq!(python_module_name(&PathBuf::from("src/__init__.py")), "src");
        assert_eq!(python_module_name(&PathBuf::from("a.py")), "a");
    }
}
