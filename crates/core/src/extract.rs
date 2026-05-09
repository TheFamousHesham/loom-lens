//! AST → graph extraction for a single parsed file.
//!
//! Produces:
//! - One `File` node and one `Module` node per file (the module's `name` is
//!   filesystem-derived at extraction time; `build.rs` rewrites it to the
//!   canonical Python import path during graph assembly).
//! - One `Function` node per top-level `def`/`async def` and per class method.
//! - One `Type` node per top-level `class`.
//! - `Contains` edges from file → module → function/type.
//! - `PendingCall` records — resolved to `Calls` edges in `build.rs`.
//! - `PendingImport` records — resolved to `Imports` edges in `build.rs`.

use crate::graph::{Edge, EdgeKind, Language, Node, NodeId, NodeKind, Span};
use crate::parser::ParsedFile;
use std::path::Path;
use tree_sitter::Node as TsNode;

/// Discriminator for the syntactic shape of a call expression. Resolution
/// rules differ per kind so the graph doesn't conflate them.
#[derive(Debug, Clone)]
pub enum CallKind {
    /// `foo(...)` — resolve to a top-level function in the same module first,
    /// then via the file's import table. Most module-level calls land here.
    Bare {
        /// Callee identifier.
        name: String,
    },
    /// `self.foo(...)` — resolve to a method of the enclosing class.
    /// Attribute calls on a non-`self` receiver are *not* recorded as
    /// `PendingCall`s at all; resolving those needs flow analysis we don't
    /// have at M1, and over-attributing to a same-named top-level function
    /// (the M1 polish bug) was worse than under-resolving.
    SelfMethod {
        /// Method name being invoked.
        method: String,
        /// `Type` node for the enclosing class.
        enclosing_class: NodeId,
    },
}

/// A call site, classified. Resolution to `NodeId` happens in `build.rs`.
#[derive(Debug, Clone)]
pub struct PendingCall {
    /// Caller function node.
    pub caller: NodeId,
    /// Source span of the call expression.
    pub site: Span,
    /// Kind discriminant.
    pub kind: CallKind,
}

/// A module-level import statement.
///
/// `from X import Y` and `from .X import Y` and `import X` all yield records
/// here, distinguished by `level` (number of leading dots; 0 = absolute) and
/// whether `name` is set.
#[derive(Debug, Clone)]
pub struct PendingImport {
    /// Importing file's node id.
    pub from_file: NodeId,
    /// Module portion of the import as written, *with no leading dots*.
    /// Empty string is legal and means `from . import X` (level >= 1, no module).
    pub module: String,
    /// Number of leading dots in a relative import. 0 = absolute (`from X import Y`).
    pub level: u32,
    /// Specific name imported, if known (`from X import Y` → `Some("Y")`;
    /// `import X` → `None`). For `from X import a, b`, one record per name.
    pub name: Option<String>,
    /// Optional alias (`from X import Y as Z` → `Some("Z")`).
    pub alias: Option<String>,
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
/// `NodeId` of the file node.
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
    /// `(class_name, class_node_id)` for the innermost enclosing `class:` block.
    current_class: Option<(String, NodeId)>,
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
        Some((cls, _)) => format!("{}::{}.{}", ctx.file_path.display(), cls, name),
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
    ctx.current_class = Some((class_name, class_id));
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
        return;
    };
    let Some(fn_expr) = node.child_by_field_name("function") else {
        return;
    };

    let kind = match fn_expr.kind() {
        "identifier" => {
            let name = node_text(fn_expr, ctx.source).to_string();
            if name.is_empty() {
                return;
            }
            CallKind::Bare { name }
        }
        "attribute" => {
            // Only record `self.foo(...)` — resolved to enclosing class methods.
            // Other receivers (`x.foo()`, `module.foo()`) drop because we have no
            // type information to know what the receiver refers to.
            let object = fn_expr.child_by_field_name("object");
            let attr = fn_expr.child_by_field_name("attribute");
            let (Some(object), Some(attr)) = (object, attr) else {
                return;
            };
            let receiver_text = node_text(object, ctx.source);
            if receiver_text != "self" {
                return;
            }
            let Some((_, class_id)) = ctx.current_class else {
                return;
            };
            let method = node_text(attr, ctx.source).to_string();
            if method.is_empty() {
                return;
            }
            CallKind::SelfMethod {
                method,
                enclosing_class: class_id,
            }
        }
        _ => return,
    };

    out.pending_calls.push(PendingCall {
        caller,
        site: span_of(node),
        kind,
    });
}

fn handle_import_statement(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    // `import a.b.c [as d], e.f` — children include dotted_name and aliased_import nodes.
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        match child.kind() {
            "dotted_name" => {
                let module = node_text(child, ctx.source).to_string();
                if module.is_empty() {
                    continue;
                }
                out.pending_imports.push(PendingImport {
                    from_file: ctx.file_id,
                    module,
                    level: 0,
                    name: None,
                    alias: None,
                    site: span_of(node),
                });
            }
            "aliased_import" => {
                let module = child
                    .child_by_field_name("name")
                    .map(|n| node_text(n, ctx.source).to_string())
                    .unwrap_or_default();
                let alias = child
                    .child_by_field_name("alias")
                    .map(|n| node_text(n, ctx.source).to_string());
                if module.is_empty() {
                    continue;
                }
                out.pending_imports.push(PendingImport {
                    from_file: ctx.file_id,
                    module,
                    level: 0,
                    name: None,
                    alias,
                    site: span_of(node),
                });
            }
            _ => {}
        }
    }
}

fn handle_import_from(node: TsNode<'_>, ctx: &mut PyCtx<'_>, out: &mut Extraction) {
    // tree-sitter-python represents `from .X import Y` as either:
    //   import_from_statement
    //     ├── '.' (one or more leading dot tokens)
    //     ├── module_name: dotted_name → 'X'
    //     └── name: dotted_name → 'Y' (one per imported name) | aliased_import
    //
    // Or, `from . import X`:
    //   import_from_statement
    //     ├── '.' (dot tokens; no module_name)
    //     └── name: dotted_name → 'X'
    //
    // We count leading dots by walking the named-and-anonymous children until we
    // hit a non-dot node.

    // tree-sitter-python's grammar puts the leading dots and module name
    // together inside the `module_name` field; the node's text reads ".X"
    // or "..X" for relative imports. For `from . import Y` (no module after
    // the dots), there is no `module_name` field and the dots appear as
    // top-level child tokens. Drive both off the literal text.

    let mut level: u32 = 0;
    let mut module = String::new();
    let mut imported: Vec<(String, Option<String>)> = Vec::new();

    if let Some(mn) = node.child_by_field_name("module_name") {
        let text = node_text(mn, ctx.source);
        let bytes = text.as_bytes();
        while (level as usize) < bytes.len() && bytes[level as usize] == b'.' {
            level += 1;
        }
        module = text[level as usize..].to_string();
    } else {
        // `from . import X` form — count dots from top-level child tokens.
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "." {
                    level += 1;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    // Collect each imported name. The grammar attaches them either as direct
    // `dotted_name` children that aren't the module_name, or as `aliased_import`.
    let mut cursor = node.walk();
    let module_name_id = node.child_by_field_name("module_name").map(|n| n.id());
    for child in node.children(&mut cursor) {
        if Some(child.id()) == module_name_id {
            continue;
        }
        match child.kind() {
            "dotted_name" => {
                let name = node_text(child, ctx.source).to_string();
                if !name.is_empty() {
                    imported.push((name, None));
                }
            }
            "aliased_import" => {
                let name = child
                    .child_by_field_name("name")
                    .map(|n| node_text(n, ctx.source).to_string())
                    .unwrap_or_default();
                let alias = child
                    .child_by_field_name("alias")
                    .map(|n| node_text(n, ctx.source).to_string());
                if !name.is_empty() {
                    imported.push((name, alias));
                }
            }
            _ => {}
        }
    }

    if imported.is_empty() {
        // Bare `from X import *` etc. — record the module-only import.
        out.pending_imports.push(PendingImport {
            from_file: ctx.file_id,
            module: module.clone(),
            level,
            name: None,
            alias: None,
            site: span_of(node),
        });
    } else {
        for (name, alias) in imported {
            out.pending_imports.push(PendingImport {
                from_file: ctx.file_id,
                module: module.clone(),
                level,
                name: Some(name),
                alias,
                site: span_of(node),
            });
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

/// Filesystem-derived module name (e.g., `src/sample_repo/__init__.py` →
/// `src.sample_repo`). `build.rs` overwrites this with the canonical Python
/// import path during graph assembly when it can determine one. Kept here as
/// a fallback for files outside any package.
pub fn python_module_name(rel: &Path) -> String {
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
        assert_eq!(
            python_module_name(&PathBuf::from("src/__init__.py")),
            "src"
        );
        assert_eq!(python_module_name(&PathBuf::from("a.py")), "a");
    }
}
