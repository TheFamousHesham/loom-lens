# TypeScript / JavaScript Effect Inference Rules

Patterns Loom Lens detects in TypeScript and JavaScript code to infer side effects. The implementation in `crates/effects/src/typescript.rs` is generated from / kept in sync with this document.

> **Status:** Drafted at Checkpoint 1. Implementation lands at Checkpoint 3 (M2).

TypeScript and JavaScript share most patterns; TypeScript adds type-annotation-based hints that increase confidence.

---

## Confidence levels

Same as Python: `definite`, `probable`, `possible`.

---

## `Net` effect

### Definite
- `fetch(...)` (browser builtin or polyfill).
- Imports + calls from these modules:
  - `axios`: `.get`, `.post`, `.put`, `.delete`, `.patch`, `.request`, `.create`
  - `node-fetch`, `cross-fetch`, `isomorphic-fetch`
  - `got`, `superagent`, `request`, `request-promise`
  - `undici`: `fetch`, `request`, `Client`, `Pool`, `Agent`
  - `http`, `https` (Node builtins): `request`, `get`, `createServer`
  - `net`, `tls` (Node builtins): `createConnection`, `Socket`, `connect`
  - `ws` (WebSocket library)
  - `@aws-sdk/*`: any client method
  - `firebase`, `@firebase/*`: database/storage/auth methods
  - `pg`, `mysql`, `mysql2`, `mongodb`, `redis`, `ioredis`: connection and query methods
  - `kafkajs`: producer/consumer methods
  - tRPC: `@trpc/client` (`createTRPCClient`, `createTRPCProxyClient`, mutations and queries)
  - GraphQL clients: `@apollo/client`, `urql`, `graphql-request`, `@tanstack/react-query` when paired with a fetch wrapper
  - Server-side data fetchers in framework code: Next.js `fetch` inside `getServerSideProps` / Server Components / `"use server"` actions, Remix `loader`/`action` functions
  - `@grpc/grpc-js` (gRPC over HTTP/2)
- `new XMLHttpRequest()`.
- `new WebSocket(...)`.
- `new EventSource(...)`.
- `navigator.sendBeacon(...)`.

### Probable
- Method calls on parameters/locals typed as `AxiosInstance`, `Client`, `Connection`, `WebSocket`, `RequestInit`.
- Function returns a `Promise<Response>` (suggests fetch-like).

### Possible
- Function name contains `fetch`, `download`, `upload`, `request`, `api`, `http`, `rpc`, `query`.
- File path matches `*api*`, `*client*`, `*http*`, `*remote*`.

---

## `IO` effect

### Definite
- Imports + calls from `fs`, `fs/promises`, `node:fs`, `node:fs/promises`:
  - `readFile`, `readFileSync`, `writeFile`, `writeFileSync`, `appendFile`, `unlink`, `rename`, `mkdir`, `rmdir`, `rm`, `chmod`, `chown`, `symlink`, `link`, `createReadStream`, `createWriteStream`, `open`.
- `fs.promises.*` equivalents.
- `path` is not IO by itself (pure manipulation), but combined with `fs` calls it's tagged at the call site.
- `console.log`, `console.warn`, `console.error`, `console.info`, `console.debug` (writes to stdout/stderr).
- `process.stdout.write`, `process.stderr.write`.
- `child_process.*`: `exec`, `execSync`, `spawn`, `spawnSync`, `fork` (also Foreign).
- `localStorage.setItem`, `localStorage.removeItem`, `localStorage.clear`, `sessionStorage.*` (browser).
- `indexedDB.*` and wrappers (`idb`, `dexie`, `localforage`) (browser).
- `document.cookie = ...` (browser).
- File System Access API: `showSaveFilePicker`, `showOpenFilePicker`, `showDirectoryPicker` (Chromium browsers).
- Origin Private File System: `navigator.storage.getDirectory()` and methods on the returned handles.

### Probable
- Method calls on objects typed as `WriteStream`, `ReadStream`, `FileHandle`.
- Function name contains `save`, `load`, `read`, `write`, `dump`, `persist`, `cache`.

### Possible
- File path suggests IO: `*storage*`, `*persist*`, `*cache*`, `*disk*`.

---

## `Mut` effect

### Definite
- Reassignment to module-scope `let`/`var` (not `const`).
- Mutation of properties on parameters: `param.x = ...`, `delete param.x`.
- Calls to mutating array methods on parameters: `.push`, `.pop`, `.shift`, `.unshift`, `.splice`, `.sort`, `.reverse`, `.fill`, `.copyWithin`.
- `Object.assign(target, ...)` where `target` is a parameter.
- Calls to mutating Map/Set methods on parameters: `.set`, `.delete`, `.clear`, `.add`.

### Probable
- Method calls on `this` after construction (in classes): `this.x = ...` outside the constructor.
- Calls to methods named `addX`, `removeX`, `setX`, `updateX`, `clear`, `reset`, `mutate`.

### Possible
- Function name starts with `add`, `remove`, `set`, `update`, `clear`, `reset`, `delete`, `mutate`.
- Function returns `void` and is not an event handler.

---

## `Throw` effect

### Definite
- `throw new SomeError(...)` or `throw expr` outside `try`.
- Re-throw inside `catch`: `catch (e) { throw e; }`.
- Calls to functions returning `never` (TypeScript) or with explicit `@throws` JSDoc.

### Probable
- Calls to known-throwing functions: `JSON.parse(...)`, array index access on `as any`, `parseInt` on user input.
- `assert(...)` calls (Node `assert` module or third-party).

### Possible
- Function name starts with `validate`, `check`, `assert`, `require`, `ensure`.
- TypeScript: function return type is `T | never` or just contains `never`.

---

## `Async` effect

### Definite
- Function is declared `async`.
- Function body contains `await`.
- Function returns `Promise<T>` (TypeScript).
- Function uses generators with `yield` inside `async function*`.

### Probable
- Function name ends with `Async`, starts with `async`.
- Function takes a callback parameter named `cb`, `callback`, `done`.

### Possible
- File path matches `*async*`, `*queue*`, `*worker*`.

---

## `Random` effect

### Definite
- `Math.random()`.
- `crypto.randomBytes`, `crypto.randomFillSync`, `crypto.randomUUID` (Node).
- `crypto.getRandomValues` (browser/Web Crypto).
- `uuid.v1`, `uuid.v3`, `uuid.v4`, `uuid.v5` (from the `uuid` package).
- Imports from `nanoid`, `cuid`, `shortid`.

### Probable
- Method calls named `.random()`, `.uuid()`, `.shuffle()`, `.choice()` on unresolved objects.

---

## `Time` effect

### Definite
- `Date.now()`, `new Date()` (without arguments — with arguments it's pure construction).
- `performance.now()`.
- `process.hrtime`, `process.hrtime.bigint`.
- `setTimeout`, `setInterval`, `setImmediate` (also `Async`).
- `clearTimeout`, `clearInterval` (Mut on the timer state, but minor — tag as `Time`).

### Probable
- Function name contains `sleep`, `wait`, `delay`, `now`, `today`, `tick`.

---

## `Foreign` effect

### Definite
- `child_process.*` (also IO).
- `worker_threads`: `Worker` constructor.
- N-API calls (rare; mostly hidden behind imports).
- `wasm-bindgen` or WebAssembly imports.
- `vm.runInThisContext`, `vm.runInNewContext`, `eval`, `new Function(...)`.
- Imports from packages with native bindings: `node-gyp`-built modules, `bcrypt`, `sharp`, `node-canvas`, `better-sqlite3`, `argon2`.

### Probable
- Dynamic `import()` with non-literal arguments.
- `require()` with non-literal arguments.

---

## TypeScript-specific bonus rules

When type information is available, confidence increases:

- A function annotated with `(): Promise<Response>` is definitely Net (fetch-like signature).
- A function annotated with `(...): void` and taking a mutable parameter (no `Readonly<T>`) is probably Mut.
- A function annotated as returning `never` is definitely Throw.
- A class extending `EventEmitter` likely has Mut effect on emit.
- A class implementing `AsyncIterable` is definitely Async.
- A method on a class decorated with `@Injectable` (Angular/Nest) is treated as a service — its effects matter for the caller graph.

---

## What this misses

- **`as` type assertions.** We can't trust user-supplied types when the asserter is wrong.
- **JSX/TSX side effects in render.** React component renders can call setState, useEffect, etc. Detection: a function returning JSX is tagged `possible Mut` if it calls hooks.
- **Decorators.** Same as Python — tag with `Foreign-possible` if unknown.
- **Re-exports and barrel files.** A function imported from `./index.ts` may forward through several modules. We follow up to 3 levels of re-export before giving up.
