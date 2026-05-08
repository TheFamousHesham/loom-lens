// Random-effect patterns.

import { randomBytes, randomUUID } from "node:crypto";

export function rollDie(): number {
  // expect: Random=definite
  return Math.floor(Math.random() * 6) + 1;
}

export function newSessionId(): string {
  // expect: Random=definite
  return randomUUID();
}

export function nonce(length: number): Buffer {
  // expect: Random=definite
  return randomBytes(length);
}

export function shuffleInPlace<T>(items: T[]): void {
  // expect: Random=definite, Mut=definite
  for (let i = items.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [items[i], items[j]] = [items[j]!, items[i]!];
  }
}

export function fakeRandom(): number {
  // expect: Random=probable
  // Method `.random()` on an unresolved object.
  const bag = { random: () => 4 };
  return bag.random();
}
