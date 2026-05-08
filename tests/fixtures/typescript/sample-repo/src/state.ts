// Mutation-effect patterns.

let counter = 0; // module-scope mutable

export function incrementGlobal(): void {
  // expect: Mut=definite
  counter += 1;
}

export function readGlobal(): number {
  // expect: (none — Pure)
  return counter;
}

export function pushItem<T>(items: T[], item: T): void {
  // expect: Mut=definite
  items.push(item);
}

export function clearList<T>(items: T[]): void {
  // expect: Mut=definite
  items.length = 0;
}

export function patchUser(user: Record<string, unknown>, changes: Record<string, unknown>): void {
  // expect: Mut=definite
  Object.assign(user, changes);
}

export class Counter {
  value = 0;

  increment(): void {
    // expect: Mut=definite
    this.value += 1;
  }
}

export function addOne(x: number): number {
  // expect: Mut=possible
  // Pure body; name starts with "add".
  return x + 1;
}

export function withItem<T>(items: ReadonlyArray<T>, item: T): T[] {
  // expect: (none — Pure)
  // Returns a new array; no mutation, no Mut tag.
  return [...items, item];
}
