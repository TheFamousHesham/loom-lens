# Python Effect Inference Rules

Patterns Loom Lens detects in Python code to infer side effects. Each pattern produces an effect tag with a confidence level. The implementation in `crates/effects/src/python.rs` is generated from / kept in sync with this document.

> **Status:** Drafted at Checkpoint 1. Implementation lands at Checkpoint 3 (M2). Refined based on real-world testing at Checkpoint 4 (M3).

This is a living document. False positives and false negatives reported via GitHub issues become rule corrections.

---

## Confidence levels

- **`definite`** — the pattern is unambiguous (e.g., `open(...)` in a builtin, `requests.get(...)` from the `requests` library imported by name).
- **`probable`** — the pattern is likely but could be shadowed (e.g., a method called `.write()` on something we can't statically resolve).
- **`possible`** — there's a reason to suspect the effect but the evidence is weak (e.g., a function whose name contains "fetch" or "load").

---

## `Net` effect

### Definite
- Calls to functions imported from these modules:
  - `requests`: `.get`, `.post`, `.put`, `.delete`, `.patch`, `.head`, `.request`, `Session`
  - `urllib.request`: `urlopen`, `urlretrieve`, `Request`
  - `urllib3`: any module-level call
  - `http.client`: `HTTPConnection`, `HTTPSConnection`
  - `httpx`: `.get`, `.post`, `.put`, `.delete`, `.patch`, `Client`, `AsyncClient`
  - `aiohttp`: `ClientSession`, `request`
  - `socket`: `socket()`, `create_connection`, `gethostbyname`
  - `ftplib`, `smtplib`, `poplib`, `imaplib`, `telnetlib`, `nntplib`
  - `boto3`, `botocore` (any client method that isn't local)
  - `redis`, `pymongo`, `psycopg2`, `psycopg`, `asyncpg`, `mysql.connector`, `pymysql`, `mysql.aio` (database calls = network in our taxonomy)
  - `kafka`, `confluent_kafka`, `aiokafka`
  - `paramiko`, `asyncssh` (SSH)
  - `grpc`, `grpcio` (gRPC)
- `__import__('socket')` etc. (dynamic import of network modules)

### Probable
- Method calls on objects whose type annotation matches `Client`, `Session`, `Connection`, `Request`, `Response` from any imported module.
- Calls to `.get(`, `.post(`, `.put(`, `.delete(` on objects we can't resolve, *if* the function has any other Net-related identifier in scope.

### Possible
- Function name contains `fetch`, `download`, `upload`, `request`, `api`, `http`, `rpc`.
- Module name matches `*api*`, `*client*`, `*http*`, `*network*`, `*remote*`.

---

## `IO` effect

### Definite
- `open(...)` (builtin) for write modes (`'w'`, `'a'`, `'x'`, `'wb'`, `'ab'`, `'r+'`, `'w+'`).
- `pathlib.Path` methods: `.write_text`, `.write_bytes`, `.unlink`, `.mkdir`, `.rmdir`, `.touch`, `.rename`, `.replace`, `.chmod`.
- `os.remove`, `os.rmdir`, `os.unlink`, `os.rename`, `os.replace`, `os.chmod`, `os.chown`, `os.makedirs`, `os.mkdir`, `os.symlink`, `os.link`.
- `os.system`, `os.popen`, `os.execv*`, `subprocess.*` (treated as IO + Foreign).
- `shutil.copy*`, `shutil.move`, `shutil.rmtree`, `shutil.make_archive`.
- `tempfile.NamedTemporaryFile`, `tempfile.mkstemp` (IO; non-Mut intermediate).
- Logging calls that go to a file handler. (Heuristic: any `logger.info`/`warning`/`error` call. Always tagged with `probable` not `definite` since logging may be in-memory.)
- `print(...)` (writes to stdout — IO). `print(..., file=...)` is also IO.
- `sys.stdout.write`, `sys.stderr.write`.
- `input(...)`, `sys.stdin.read`, `sys.stdin.readline` (reading stdin — IO).
- Disk-backed serialization round-trips: `marshal.load`, `shelve.open`, `dbm.open`, and the file-reading variants of the standard `pickle` module (used for deserialization from a path/file). These are detected as IO regardless of any orthogonal security concerns.

### Probable
- `open(...)` for read modes (`'r'`, `'rb'`) — read is also IO in our taxonomy.
- Method calls on objects of type `IO`, `TextIO`, `BinaryIO`, `BufferedReader`, `BufferedWriter`, `Path`.
- `.read()`, `.write()`, `.close()` on unresolved objects when there's an `open()` call earlier in the function.

### Possible
- Function name contains `save`, `load`, `read`, `write`, `dump`, `parse_file`.
- Calls to functions with `path`, `file`, `filename` parameters.

---

## `Mut` effect

### Definite
- Assignment to module-level (global) variables: `global x; x = ...` or top-level `x = ...` followed by `x = ...` inside a function.
- Calls to `list.append`, `list.extend`, `list.insert`, `list.pop`, `list.remove`, `list.clear`, `list.sort`, `list.reverse`, `dict.update`, `dict.pop`, `dict.clear`, `set.add`, `set.remove`, `set.discard`, `set.update`, `set.clear` on parameters of the function (not on locals — locals don't escape).
- Setting attributes on `self` after `__init__` returns. (Detection: any `self.x = ...` outside `__init__` is `Mut`.)
- Setting attributes on parameters: `param.x = ...`.
- `del` on parameters or globals.

### Probable
- Calls to methods commonly named to indicate mutation: `add_*`, `remove_*`, `update_*`, `set_*`, `clear`, `reset`, `delete_*`.

### Possible
- Function name starts with `update_`, `set_`, `add_`, `remove_`, `clear_`, `reset_`, `delete_`.
- Function returns `None` and isn't a callback or main entry point. (Mutation is the most common reason a Python function returns nothing.)

---

## `Throw` effect

### Definite
- Bare `raise SomeException(...)` or `raise SomeException` statements without a surrounding `try`.
- `raise` re-raise inside an `except` block (propagates).
- Any function that calls another function with a `Throw` effect, *if* the call site is not inside `try`/`except`.

### Probable
- Calls to functions whose docstring mentions "raises" or "throws".
- Built-in functions known to raise: `int(...)` on non-numeric, `dict[key]` access, `list[i]` access.
- `assert ...` statements (raise `AssertionError` when the condition is falsy and `python -O` is not in effect).

### Possible
- Function name starts with `validate_`, `check_`, `assert_`, `require_`, `ensure_`.

---

## `Async` effect

### Definite
- The function is declared with `async def`.
- The function body contains `await ...`.
- The function calls `asyncio.run`, `asyncio.create_task`, `asyncio.gather`, `asyncio.wait`, `loop.run_*`.
- Imports from `asyncio`, `trio`, `anyio`, `curio`.

### Probable
- Function name ends with `_async` or starts with `async_`.

### Possible
- Function name suggests concurrency: `worker`, `task`, `job`, `runner`.

---

## `Random` effect

### Definite
- Calls to `random.*` (any function from the stdlib `random` module).
- Calls to `secrets.*`.
- Calls to `numpy.random.*`.
- Calls to `uuid.uuid1`, `uuid.uuid3`, `uuid.uuid4`, `uuid.uuid5`.
- Calls to `os.urandom`, `os.getrandom`.

### Probable
- Calls to methods named `.choice`, `.sample`, `.shuffle`, `.random` on unresolved objects.

---

## `Time` effect

### Definite
- Calls to `time.time`, `time.monotonic`, `time.perf_counter`, `time.process_time`, `time.sleep`, `time.gmtime`, `time.localtime`.
- Calls to `datetime.datetime.now`, `datetime.datetime.utcnow`, `datetime.datetime.today`, `datetime.date.today`.
- Calls to `asyncio.sleep`.

### Probable
- Function name contains `sleep`, `wait`, `delay`, `timeout`, `now`, `today`, `current_time`.

---

## `Foreign` effect

### Definite
- `import ctypes`, `import cffi`.
- Calls to `ctypes.CDLL`, `ctypes.PyDLL`, `cffi.FFI`.
- Calls to `subprocess.*` (also IO).
- Calls to `os.system`, `os.popen`.
- Imports from C extension modules (heuristic: module from a `.so`/`.pyd` file).

### Probable
- Imports from packages known to wrap native libraries: `numpy`, `scipy`, `pandas`, `pillow`, `cryptography`, `lxml`, `psycopg2`, `mysqlclient`. (Tag the *imports* as `Foreign-probable`; calls don't get tagged unless they're known to invoke native code.)

---

## What this misses (known limitations)

- **Decorators that change behavior.** `@cached`, `@retry`, `@timed`, etc., may add or remove effects. Detection: if a function has decorators we don't recognize, tag with `Foreign-possible`.
- **Metaclasses.** Same problem; tag the class with `Foreign-possible`.
- **Dynamic imports.** `importlib.import_module(name)` where `name` is a variable — we can't know what's imported. Tag the call with `Foreign-definite`.
- **`exec` and `eval`.** Always tagged `Foreign-definite + IO-possible + Net-possible + Mut-possible` (anything could happen).
- **Type-stub-only annotations.** If we don't have a body, we can't infer.
