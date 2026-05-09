# 0002 — Effect Inference: Heuristic, Per-Language, Confidence-Aware

**Date:** Pre-Checkpoint 1
**Status:** Accepted (2026-05-09)
**Reviewed at checkpoint:** 1 ✓

## Context

Mode 2 of Loom Lens identifies side effects in code: which functions hit the network, mutate state, throw, etc. Two fundamentally different approaches exist:

- **Sound static analysis** — provably correct effect identification using formal techniques (abstract interpretation, type-and-effect systems, theorem provers). Requires a typed program representation, language-by-language formal modeling, often years of research.
- **Heuristic pattern matching** — match known idioms in the AST ("`fetch(...)` indicates Net effect"). Fast to build, language-agnostic structure, well-understood failure modes. Will have false positives and false negatives.

Loom Lens is a 4-week project. Sound analysis is out of scope. The question is whether heuristic analysis is *useful* despite being unsound.

## Decision

**Heuristic, per-language, with explicit confidence levels surfaced in the UI.**

- Per-language rules in `docs/effect-rules/{python,typescript,rust}.md` describe the patterns to detect.
- Each detection assigns a confidence: `definite`, `probable`, `possible`.
- Confidence shown visually in the viewer (solid color / striped / outline).
- Hover state shows the evidence ("Net inferred from `fetch()` at line 42").
- Effects propagate transitively through the intra-repo call graph.
- Calls into external libraries are tagged `External` rather than expanded into their effects.

## Alternatives considered

- **Sound static analysis.** Out of scope as discussed.
- **Single confidence level (just "has effect" / "doesn't").** Rejected. Users will distrust a tool that confidently mislabels code; explicit confidence preserves trust by being honest about uncertainty.
- **Effect inference per import statement only** (e.g., "imports `requests` → marked Net"). Easier to implement but produces too many false positives — importing a library doesn't mean every function uses it.
- **Language-agnostic rules.** Rejected. Effect patterns are deeply language-specific; trying to share rules across Python and Rust will produce nothing useful for either.
- **Wait for the user to write rules manually.** Rejected. The default rules need to ship working out of the box; user customization is a v0.2 feature.

## Consequences

### Positive
- Tractable in 4 weeks.
- Patterns are easy to add — community can contribute new ones.
- Confidence levels build trust honestly. Users learn what the tool is good at.
- Per-language rule files double as documentation of "what the tool actually detects."

### Negative
- Will have false positives and false negatives. We must communicate this clearly.
- Quality is bounded by rule coverage. A library that uses unusual patterns will be misanalyzed.
- Rules drift over time as language idioms change. Maintenance burden.

### Risks
- Users may treat the lens output as authoritative ("Loom Lens says this function is pure, so it must be"). Mitigation: prominent UI disclaimers; confidence indicators; "show evidence" affordance on every effect tag.
- Languages with strong macro/metaprogramming (Rust macros, Python decorators) hide effects from AST inspection. Mitigation: tag macro/decorator calls as "may affect this analysis" with `Foreign`-confidence.

## References

- Why heuristic effect analysis is useful even though unsound: discussion in any paper on "lightweight verification."
- Existing tools with similar approaches: SonarQube (security hotspots, not formally sound), CodeQL (more rigorous but configurable), various IDE inspectors.

## Refinements at Checkpoint 1

The 9-effect taxonomy stands. The following operational details were nailed down so M2 can be implemented without re-litigating them:

- **Effect set, locked.** `Pure`, `Net`, `IO`, `Mut`, `Throw`, `Async`, `Random`, `Time`, `Foreign`. Pure is the *absence* of evidence, not a positive claim — we do not infer purity, we infer the absence of detectable impurity. UI surfaces this distinction in tooltips ("no detected effects" rather than "pure").
- **`Result`/`Option` is *not* an effect.** In Rust, returning `Result<T, E>` is not flagged as `Throw`; only paths that can `panic!`/`unwrap` are. The earlier note in `documentation/docs/effect-rules/rust.md` ("Decision pending in this ADR") is hereby decided: **`Result`-returning is a control-flow shape, not an effect.** Functions that propagate errors via `?` inherit `Throw` only if a callee can panic.
- **Effect aggregation rule.** A function carries the *union* of its body's detected effects and (for direct intra-repo calls) the union of its callees' effects. Confidence aggregation: when multiple matches imply the same effect, the strongest confidence wins (definite > probable > possible). When transitively inherited, confidence is one level weaker than the source (definite-in-callee → probable-on-caller, probable-in-callee → possible-on-caller). This keeps transitive labeling honest about the loss of precision.
- **Recursion / SCCs.** For mutually-recursive call cycles, compute effects on the strongly-connected component as a fixpoint: every node in the SCC gets the union of effects of every node in the SCC. This is the standard approach and avoids order-dependence.
- **`External` tag is its own dimension.** Calls into external libraries are marked `External` *in addition to* whatever effects we can attribute. `External` is a provenance tag on the edge, not a member of the effect set itself. The UI offers a per-effect filter "include external calls?" defaulting off.
- **Confidence-level UI mapping, locked.** `definite` → solid color, `probable` → diagonal-stripe pattern, `possible` → outline only. This is the contract; viewer-mockup.md depicts it.
- **Adding/removing effects is itself an architectural change.** New language support may need to extend the rule set (e.g., Go `goroutine` for Async); but introducing a new effect category (e.g., `Crypto`, `IPC`) requires an amendment to this ADR or a superseding one.
- **Per-language rules are normative, not advisory.** `documentation/docs/effect-rules/{python,typescript,rust}.md` define what the implementation does. When the implementation diverges, fix the implementation. Rule additions go in the document, then in the code.
