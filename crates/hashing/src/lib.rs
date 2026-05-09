//! Loom Lens hashing. M1 wires BLAKE3 over raw bytes for the file-set
//! component of the `graph_id`; AST normalization for per-function hashes
//! lands at M3 (per ADR 0004 refinements).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use blake3::Hasher;

/// Compute a graph_id from a canonical path and a sorted list of per-file
/// content hashes. See ADR 0004 §"Refinements at Checkpoint 1".
#[must_use]
pub fn graph_id_for(repo_root: &str, file_hashes: &[String]) -> String {
    let mut h = Hasher::new();
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

/// Hex-encoded BLAKE3 of a byte slice.
#[must_use]
pub fn blake3_hex(bytes: &[u8]) -> String {
    let h = blake3::hash(bytes);
    hex::encode(h.as_bytes())
}
