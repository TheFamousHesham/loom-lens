# Rust Effect Inference Rules

Patterns Loom Lens detects in Rust code to infer side effects. The implementation in `crates/effects/src/rust.rs` is generated from / kept in sync with this document.

> **Status:** Drafted at Checkpoint 1. Implementation lands at Checkpoint 3 (M2).

Rust's type system already encodes some effects (`Result` for fallibility, `async fn` for async, `&mut` for mutation), so inference is more reliable than for Python or TypeScript — but Rust also has macros that hide effects.

---

## Confidence levels

Same: `definite`, `probable`, `possible`.

---

## `Net` effect

### Definite
- Calls into these crates:
  - `reqwest`: `get`, `post`, `Client`, `blocking::*`
  - `hyper`: `Client`, `Server`, request/response types
  - `tokio::net`: `TcpStream`, `TcpListener`, `UdpSocket`, `UnixStream`, `UnixListener`
  - `std::net`: `TcpStream`, `TcpListener`, `UdpSocket`
  - `axum`, `warp`, `actix-web`, `rocket`, `tide`, `poem`: route handlers and client calls
  - `surf`, `ureq`, `isahc`, `attohttpc`
  - `tokio-tungstenite`, `tungstenite`, `async-tungstenite` (WebSocket)
  - `quinn` (QUIC), `webrtc-rs`
  - `aws-sdk-*` (any AWS SDK crate)
  - `redis`, `mongodb`, `tokio-postgres`, `sqlx::postgres::*`, `sqlx::mysql::*`, `mysql_async` — database clients (Net in our taxonomy)
  - `lapin` (AMQP), `rdkafka`

### Probable
- Calls on values of type `&Client`, `&mut Client`, `Connection`, `Stream` from any imported crate.
- Function returns `impl Future<Output = Result<Response, _>>`.

### Possible
- Function name contains `fetch`, `download`, `upload`, `request`, `api`, `http`, `rpc`, `query`, `connect`.
- Module name matches `*api*`, `*client*`, `*http*`, `*remote*`, `*net*`.

---

## `IO` effect

### Definite
- Calls on `std::fs`: `read`, `write`, `read_to_string`, `read_to_end`, `File::open`, `File::create`, `OpenOptions::*`, `remove_file`, `remove_dir`, `remove_dir_all`, `rename`, `copy`, `metadata`, `set_permissions`, `create_dir`, `create_dir_all`, `hard_link`, `soft_link`, `symlink_*`.
- Calls on `tokio::fs::*` (async equivalents).
- `println!`, `print!`, `eprintln!`, `eprint!`, `dbg!` macros.
- `std::io::stdout()`, `std::io::stderr()`.
- `std::process::Command::*` (also Foreign).
- Calls on `Read` and `Write` trait methods of `&File`, `&TcpStream`, etc.
- `tracing` and `log` macros that go through a configured subscriber/logger (also see `Log` if we add that effect category).

### Probable
- Method calls on values of type `File`, `BufReader<_>`, `BufWriter<_>`, `Stdout`, `Stderr`.
- Function takes a `&Path` or `PathBuf` parameter and isn't pure path manipulation.

### Possible
- Function name contains `save`, `load`, `read_*`, `write_*`, `dump_*`, `persist`.

---

## `Mut` effect

### Definite (this is the strong case in Rust — the compiler tells us)
- Function takes any `&mut T` parameter.
- Function body assigns to a static mutable: `unsafe { STATIC_VAR = ... }`.
- Function takes a parameter with interior-mutability types (`Cell<T>`, `RefCell<T>`, `Mutex<T>`, `RwLock<T>`, `AtomicU*`, `AtomicI*`, `AtomicBool`, `AtomicPtr`) and calls a mutation method on them.
- Method on `&mut self`.

### Probable
- Function takes `Arc<Mutex<T>>` or `Arc<RwLock<T>>` and locks it. (We treat lock-and-mutate as Mut even if we can't confirm the mutation, because the standard pattern is to mutate after locking.)
- Calls to known-mutating methods on parameters: `.push`, `.insert`, `.remove`, `.clear`, `.extend`, `.append`, `.drain`, `.sort`, `.reverse`, `.swap`, `.replace`, `.take`.

### Possible
- Function name contains `add`, `remove`, `set`, `update`, `clear`, `reset`, `delete`, `mutate`.

---

## `Throw` effect

In Rust, "throw" maps to `panic!`, `unwrap`/`expect` failures, and to `?` propagation of `Result::Err`. We tag these distinctly:

- `Throw` = could panic
- `Result` types are tracked as a separate concept (`Fallible`?) — we may or may not include this in the effect taxonomy. **Decision pending in this ADR; default to including `Result`-returning as a non-effect for now and revisit if users want it.**

### Definite (panic)
- Body contains `panic!`, `unreachable!`, `unimplemented!`, `todo!`.
- Body contains `.unwrap()`, `.expect(...)`, `.unwrap_err()`, `.expect_err(...)`.
- Body contains array indexing `arr[i]` (can panic on out-of-bounds).
- Body contains division by a non-literal value (can panic on divide-by-zero in debug; wraps in release).
- Calls to functions whose body has `Throw-definite` (transitive).

### Probable
- Calls to functions that return `T` (not `Option<T>`/`Result<T, _>`) but accept user input. Heuristic only.

### Possible
- Function name starts with `validate_`, `check_`, `assert_`, `require_`, `ensure_`.

---

## `Async` effect

### Definite
- Function is `async fn`.
- Function returns `impl Future<Output = _>` or `BoxFuture<_>` or any `Future`-implementing type.
- Function body uses `.await` (closures inside the function don't count for the function itself; they count for the closure).
- Calls to `tokio::spawn`, `async_std::task::spawn`, `tokio::join!`, `tokio::select!`.

### Probable
- Function name ends with `_async`, starts with `async_`.

### Possible
- Module name matches `*async*`, `*futures*`, `*stream*`.

---

## `Random` effect

### Definite
- Calls into `rand`, `rand_core`, `rand_chacha`, `getrandom` crates.
- `rand::random()`, `rand::thread_rng()`, `rand::rngs::*`.
- `uuid::Uuid::new_v4()`, `Uuid::now_v7`.
- `nanoid::nanoid!`.

### Probable
- Method calls named `.gen()`, `.gen_range()`, `.choose()`, `.shuffle()`, `.sample()` on unresolved types.

---

## `Time` effect

### Definite
- `std::time::Instant::now()`.
- `std::time::SystemTime::now()`.
- `chrono::Utc::now`, `chrono::Local::now`.
- `time::OffsetDateTime::now_utc`, `time::Instant::now`.
- `std::thread::sleep`, `tokio::time::sleep`, `tokio::time::interval`.

### Probable
- Function name contains `now`, `sleep`, `wait`, `delay`, `tick`.

---

## `Foreign` effect

This is significant in Rust. `unsafe` blocks and FFI deserve careful tagging.

### Definite
- Function body contains an `unsafe` block.
- Function is declared `unsafe fn`.
- `extern "C"` declarations (FFI).
- Calls to `extern` functions.
- `std::mem::transmute`, `std::mem::transmute_copy`.
- `std::ptr::*` raw pointer manipulation.
- Imports from `libc`, `winapi`, `core_foundation`.
- Inline assembly: `asm!`, `global_asm!`.

### Probable
- Calls to functions known to wrap FFI: most of `tokio::net`, anything in `mio`, anything in `nix`.
- Crate dependencies on `*-sys` crates.

---

## Macro handling

Rust macros can do anything. Our approach:

- Known stdlib macros (`println!`, `vec!`, `format!`, etc.) are handled directly with their effects.
- Known third-party macros (`tracing::info!`, `serde::Serialize`, `tokio::main`, etc.) have hardcoded effect mappings in the rule file.
- Unknown macros are tagged `Foreign-possible` on the calling function.

This is a limitation; users with macro-heavy code will see less precise analysis. M3 should explore Tree-sitter's ability to expand certain macros for better fidelity.

---

## What this misses

- **Trait method dispatch.** When a method is called on a `dyn Trait`, we don't know the concrete type. We tag the call as `Foreign-possible` and use the trait's documentation as a hint if available.
- **Generic functions.** Effects depend on the concrete instantiation. We analyze the bound functions and propagate effects along monomorphizations.
- **`build.rs` scripts.** Build scripts run at compile time and have IO and Foreign effects but aren't runtime code. We skip them.
- **`#[cfg]`-gated code.** Effects may only apply on certain platforms. We analyze all configurations and union the effects (over-approximation, intentional).
