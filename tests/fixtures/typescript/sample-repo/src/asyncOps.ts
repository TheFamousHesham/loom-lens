// Async-effect patterns.

export async function fetchAll(urls: string[]): Promise<string[]> {
  // expect: Net=definite, Async=definite
  return Promise.all(urls.map(async (u) => (await fetch(u)).text()));
}

export async function delay(ms: number): Promise<void> {
  // expect: Async=definite, Time=definite
  await new Promise((r) => setTimeout(r, ms));
}

export async function* streamLines(): AsyncGenerator<string> {
  // expect: Async=definite
  yield "a";
  yield "b";
}

export function workerLoop(): void {
  // expect: Async=possible
  // Pure body; name pattern only.
  return;
}

export function returnsPromise(): Promise<number> {
  // expect: Async=definite
  // Function annotated as returning Promise<T>.
  return Promise.resolve(42);
}
