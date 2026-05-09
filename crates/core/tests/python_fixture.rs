//! Integration test: analyze the Python fixture and check shape.

use loom_lens_core::{analyze_repo, DiscoveryOpts, NodeKind};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/python/sample-repo")
        .canonicalize()
        .expect("python fixture missing")
}

#[test]
fn parses_python_fixture_with_meaningful_shape() {
    let opts = DiscoveryOpts::default();
    let g = analyze_repo(&fixture_root(), &opts).expect("analyze_repo failed");

    // The fixture has 12 .py files (10 modules + __init__ + maybe pyproject not py).
    assert!(g.summary.files >= 10, "files: {:?}", g.summary.files);
    assert!(g.summary.functions >= 30, "functions: {}", g.summary.functions);
    assert!(g.summary.languages.contains_key("python"));

    // Spot-check that fetch_user shows up.
    let fetch_user = g.nodes.iter().find(|n| {
        matches!(&n.kind, NodeKind::Function { qualified_name, .. } if qualified_name.ends_with("net.py::fetch_user"))
    });
    assert!(fetch_user.is_some(), "fetch_user not found");

    // The graph_id is content-addressed — re-running on the same content
    // should produce the same id.
    let g2 = analyze_repo(&fixture_root(), &opts).unwrap();
    assert_eq!(g.graph_id, g2.graph_id);
}
