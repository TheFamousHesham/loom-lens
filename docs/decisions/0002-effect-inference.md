# 0002 — Effect Inference: Heuristic, Per-Language, Confidence-Aware

**Date:** Pre-Checkpoint 1
**Status:** Proposed
**Reviewed at checkpoint:** 1 (pending)

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
