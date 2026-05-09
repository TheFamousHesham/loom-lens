//! Loom Lens effect inference. M1 ships an empty stub: every function is
//! tagged as `Pure` (absence of evidence) until the per-language rule
//! engines land at M2 (per `documentation/docs/effect-rules/`).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use serde::{Deserialize, Serialize};

/// Effect kinds. See ADR 0002 for the locked taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Effect {
    /// Network I/O.
    Net,
    /// Filesystem / stdout / stderr.
    Io,
    /// Non-local mutation.
    Mut,
    /// Panics / unhandled throws.
    Throw,
    /// Async-runtime usage.
    Async,
    /// Non-deterministic randomness.
    Random,
    /// Clock reads, sleeps.
    Time,
    /// FFI / unsafe / native bindings.
    Foreign,
}

/// Confidence levels (per ADR 0002).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// Outline-only in the UI.
    Possible,
    /// Striped fill.
    Probable,
    /// Solid fill.
    Definite,
}

/// One inferred effect on a function.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectTag {
    /// Effect identity.
    pub effect: Effect,
    /// Confidence level.
    pub confidence: Confidence,
    /// Human-readable evidence ("`fetch()` at line 45").
    pub evidence: String,
}

/// Run inference on a parsed graph. M1 is a no-op; M2 wires real rules.
#[must_use]
pub fn infer(_graph: &loom_lens_core::CodeGraph) -> Vec<(loom_lens_core::NodeId, Vec<EffectTag>)> {
    Vec::new()
}
