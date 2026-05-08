# Rust Effect-Inference Fixture

Source code crafted to exercise the rules in `documentation/docs/effect-rules/rust.md`. Annotation convention matches the Python and TypeScript fixtures: every function has a `// expect: ...` line below the signature.

> **Build status: intentionally non-compiling.** `cargo check` will fail because external crates referenced via `use` (`reqwest`, `tokio`, `sqlx`, etc.) are not declared as dependencies. The fixture's purpose is to be parsed by Tree-sitter, not built. M2 effect-inference tests load these files directly via the parser.

## Module map

| File | Primary effects exercised |
|------|---------------------------|
| `src/net.rs` | Net |
| `src/io_ops.rs` | IO |
| `src/state.rs` | Mut |
| `src/errors.rs` | Throw (panic) |
| `src/async_ops.rs` | Async |
| `src/randomness.rs` | Random |
| `src/clock.rs` | Time |
| `src/foreign.rs` | Foreign (unsafe, FFI) |
| `src/pure_fn.rs` | Pure (control case) |
| `src/duplicates.rs` | Hash equivalence |
| `src/false_positives.rs` | Name patterns without underlying effects |

`src/lib.rs` re-exports the modules.
