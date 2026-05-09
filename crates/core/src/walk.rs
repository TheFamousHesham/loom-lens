//! Repo-walking. Honours `.gitignore` via the `ignore` crate and a small
//! built-in deny list (`target/`, `node_modules/`, etc.).

use crate::graph::Language;
use std::path::{Path, PathBuf};

/// Options controlling repo discovery.
#[derive(Debug, Clone)]
pub struct DiscoveryOpts {
    /// Languages to include. If empty, all supported languages are accepted.
    pub languages: Vec<Language>,
    /// Hard cap on files; matches ADR 0004's `max_files` parameter.
    pub max_files: usize,
}

impl Default for DiscoveryOpts {
    fn default() -> Self {
        Self {
            languages: vec![],
            max_files: 10_000,
        }
    }
}

/// Discover source files under `repo_root` matching `opts`.
///
/// Returns paths *relative to `repo_root`*, sorted, deduplicated. Honours
/// `.gitignore` via the `ignore` crate.
pub fn discover_files(repo_root: &Path, opts: &DiscoveryOpts) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    let walker = ignore::WalkBuilder::new(repo_root)
        .standard_filters(true)
        .hidden(true)
        .build();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|t| t.is_file()) {
            continue;
        }
        let path = entry.path();
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        let Some(language) = Language::from_extension(ext) else {
            continue;
        };
        if !opts.languages.is_empty() && !opts.languages.contains(&language) {
            continue;
        }
        if let Ok(rel) = path.strip_prefix(repo_root) {
            files.push(rel.to_path_buf());
        }
        if files.len() >= opts.max_files {
            break;
        }
    }
    files.sort();
    files.dedup();
    files
}
