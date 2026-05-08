// Functions whose names look like effects but whose bodies are pure.
// These exercise the `possible` confidence level only.

export function fetchDefaultColor(): string {
  // expect: Net=possible
  return "blue";
}

export function saveToMemory<T>(items: ReadonlyArray<T>, item: T): T[] {
  // expect: IO=possible, Mut=possible
  // Returns a new array; original `items` is not mutated.
  return [...items, item];
}

export function updateLabel(text: string): string {
  // expect: Mut=possible
  return text.toUpperCase();
}

export function nowStr(): string {
  // expect: Time=possible
  return "now";
}

export function downloadName(_url: string): string {
  // expect: Net=possible
  return "filename.txt";
}
