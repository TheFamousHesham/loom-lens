# TypeScript Effect-Inference Fixture

Source code crafted to exercise the rules in `documentation/docs/effect-rules/typescript.md`. Annotation convention matches the Python fixture: every function has a `// expect: ...` line below the signature describing the expected output.

## Module map

| Module | Primary effects exercised |
|--------|---------------------------|
| `src/net.ts` | Net |
| `src/io.ts` | IO |
| `src/state.ts` | Mut |
| `src/errors.ts` | Throw |
| `src/asyncOps.ts` | Async |
| `src/randomness.ts` | Random |
| `src/clock.ts` | Time |
| `src/foreignBindings.ts` | Foreign |
| `src/pure.ts` | Pure (control case) |
| `src/duplicates.ts` | Hash equivalence across renames |
| `src/falsePositives.ts` | Name patterns without underlying effects |

The repo intentionally does *not* include `node_modules/`. Type information is approximated via `.d.ts`-style comment annotations rather than installed packages, since the engine reads source — not the installed dependency tree.
