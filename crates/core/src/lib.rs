//! Loom Lens core: parsing and code-graph extraction.
//!
//! Public surface stabilised at M1; types here are referenced by the
//! `mcp`, `viewer`, `effects`, and `hashing` crates. See
//! `documentation/ARCHITECTURE.md` §4 for the data model.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod graph;
pub mod parser;
pub mod walk;

pub use graph::{
    CodeGraph, Edge, EdgeKind, GraphId, Language, Node, NodeId, NodeKind, Span, Summary,
};
pub use parser::{parse_file, ParseError, ParsedFile};
pub use walk::{discover_files, DiscoveryOpts};

/// The library's error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// I/O failure while reading the repo.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    /// A file failed to parse.
    #[error("parse: {0}")]
    Parse(#[from] parser::ParseError),
    /// JSON ser/de failure (graph snapshot).
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
}

/// Convenience alias used across the crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;
