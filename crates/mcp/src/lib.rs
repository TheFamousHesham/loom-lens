//! Loom Lens MCP server (stdio JSON-RPC 2.0).
//!
//! Hand-rolled JSON-RPC; the protocol surface is small enough that a 200-LOC
//! implementation is more legible than a third-party SDK whose maturity is
//! still in flux. See ADR 0001 §"Refinements at Checkpoint 1".
//!
//! Wire-up in this M1 commit is intentionally minimal:
//! - `initialize` returns server info + capabilities.
//! - `tools/list` returns the four tool definitions from `documentation/docs/api.md`.
//! - `tools/call` dispatches `analyze_repo`; the other three return
//!   `MethodNotFound` until M2/M3.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod protocol;
pub mod server;
pub mod tools;

pub use server::{run_stdio, ServerState};
