// Loom Lens Rust fixture crate.
// See README.md — this crate intentionally does not compile.

#![allow(unused, dead_code, unused_imports, missing_docs, clippy::all)]

pub mod async_ops;
pub mod clock;
pub mod duplicates;
pub mod errors;
pub mod false_positives;
pub mod foreign;
pub mod io_ops;
pub mod net;
pub mod pure_fn;
pub mod randomness;
pub mod state;
