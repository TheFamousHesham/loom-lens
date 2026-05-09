//! Parser scaffolding. Real Python AST → graph extraction lands in the next pass.
//!
//! The current pass only validates that tree-sitter and tree-sitter-python
//! link in cleanly and that we can parse a buffer to a syntax tree without
//! diagnostic errors. Graph extraction (functions, calls, imports) is the
//! next commit.

use crate::graph::Language;
use std::path::{Path, PathBuf};
use tree_sitter::{Parser, Tree};

/// Errors produced by the parser layer.
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// Tree-sitter could not load a language grammar.
    #[error("language load failed for {0:?}: {1}")]
    LanguageLoad(Language, String),
    /// File contents could not be read.
    #[error("read {path:?}: {source}")]
    Read {
        /// File path that failed.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// Tree-sitter returned an error tree (one or more ERROR nodes).
    #[error("syntax errors in {path:?} starting at line {line}")]
    Syntax {
        /// File path with the syntax error.
        path: PathBuf,
        /// 1-based line of the first error.
        line: u32,
    },
}

/// A successfully-parsed source file.
#[derive(Debug)]
pub struct ParsedFile {
    /// File path (as discovered).
    pub path: PathBuf,
    /// Detected language.
    pub language: Language,
    /// Raw bytes (kept for source-span resolution).
    pub source: Vec<u8>,
    /// Tree-sitter parse tree.
    pub tree: Tree,
}

/// Parse a single file. Currently only Python is wired up; other languages
/// return `LanguageLoad` until grammars are added at M2.
pub fn parse_file(path: &Path) -> Result<ParsedFile, ParseError> {
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let language = Language::from_extension(extension).ok_or_else(|| {
        ParseError::LanguageLoad(
            Language::Python,
            format!("unrecognised extension {extension:?}"),
        )
    })?;

    let source = std::fs::read(path).map_err(|source| ParseError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    let mut parser = Parser::new();
    let grammar = match language {
        Language::Python => tree_sitter_python::language(),
        Language::Typescript | Language::Javascript | Language::Rust => {
            return Err(ParseError::LanguageLoad(
                language,
                "grammar not yet wired (M2)".into(),
            ));
        }
    };
    parser
        .set_language(&grammar)
        .map_err(|e| ParseError::LanguageLoad(language, e.to_string()))?;

    let tree = parser.parse(&source, None).ok_or_else(|| ParseError::Syntax {
        path: path.to_path_buf(),
        line: 0,
    })?;

    if tree.root_node().has_error() {
        let line = tree.root_node().start_position().row as u32 + 1;
        return Err(ParseError::Syntax {
            path: path.to_path_buf(),
            line,
        });
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language,
        source,
        tree,
    })
}
