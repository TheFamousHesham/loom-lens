# Python Effect-Inference Fixture

Source code crafted to exercise the rules in `documentation/docs/effect-rules/python.md`. The inference engine in `crates/effects/src/python.rs` is tested against this repo at M2.

Each module isolates one or two effect categories. Functions are annotated with the *expected* output via a `# expect: ...` comment immediately below the signature, in this format:

```
# expect: Net=definite, Throw=probable
```

A function with no `# expect:` line is expected to be `Pure` (absence of detected effects).

## Module map

| Module | Primary effects exercised |
|--------|---------------------------|
| `net.py` | Net |
| `io_ops.py` | IO |
| `state.py` | Mut |
| `errors.py` | Throw |
| `async_ops.py` | Async |
| `randomness.py` | Random |
| `clock.py` | Time |
| `native.py` | Foreign (+ IO) |
| `pure.py` | Pure (control case) |
| `duplicates.py` | Hash-equality across renames; near-duplicates |
| `false_positives.py` | Name patterns that look like effects but aren't |

The repo is deliberately a graph, not a flat list: `net.py::fetch_user` calls `errors.py::raise_if_invalid` which calls `state.py::record_error`. Transitive effect propagation is tested by the chain.
