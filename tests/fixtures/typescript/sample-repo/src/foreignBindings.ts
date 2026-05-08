// Foreign-effect patterns.
//
// Note: child_process invocations (e.g., spawnSync) and dynamic-code-evaluation
// patterns (e.g., the `new Function(...)` constructor and direct eval) also
// belong here as Foreign. They are omitted from this fixture because the dev
// environment's safety hooks reject the literal invocation in source files.
// The python fixture covers the subprocess case; the rule for the eval/Function
// family still applies to TS and is documented in
// `documentation/docs/effect-rules/typescript.md`.

import { Worker } from "node:worker_threads";

export function spawnWorker(scriptPath: string): Worker {
  // expect: Foreign=definite
  return new Worker(scriptPath);
}

export function dynamicRequire(modulePath: string): unknown {
  // expect: Foreign=probable
  // require() with a non-literal argument.
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  return require(modulePath);
}

export async function loadModule(modulePath: string): Promise<unknown> {
  // expect: Foreign=probable, Async=definite
  // Dynamic import() with a non-literal argument.
  return import(modulePath);
}
