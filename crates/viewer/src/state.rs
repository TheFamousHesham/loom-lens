//! Viewer state — an LRU of analysed graphs.

use indexmap::IndexMap;
use loom_lens_core::{CodeGraph, GraphId};
use std::sync::{Arc, Mutex};

/// LRU cache size, per ADR 0004.
const LRU_MAX: usize = 8;

/// Shared state held by axum handlers.
#[derive(Clone, Default)]
pub struct ViewerState {
    inner: Arc<Mutex<Inner>>,
}

#[derive(Default)]
struct Inner {
    graphs: IndexMap<GraphId, CodeGraph>,
}

impl ViewerState {
    /// Empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert (or replace) a graph and evict the oldest if needed.
    pub fn put(&self, graph: CodeGraph) {
        let mut inner = self.inner.lock().expect("poisoned");
        if inner.graphs.contains_key(&graph.graph_id) {
            inner.graphs.shift_remove(&graph.graph_id);
        }
        if inner.graphs.len() >= LRU_MAX {
            inner.graphs.shift_remove_index(0);
        }
        inner.graphs.insert(graph.graph_id.clone(), graph);
    }

    /// Look up a graph by id.
    #[must_use]
    pub fn get(&self, id: &GraphId) -> Option<CodeGraph> {
        self.inner.lock().expect("poisoned").graphs.get(id).cloned()
    }
}
