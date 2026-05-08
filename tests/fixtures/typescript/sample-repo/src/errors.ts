// Throw-effect patterns.

export class InvalidId extends Error {}

export function raiseIfInvalid(id: number): void {
  // expect: Throw=definite
  if (id < 0) {
    throw new InvalidId(`negative id: ${id}`);
  }
}

export function reraise(value: string): number {
  // expect: Throw=definite
  try {
    return Number.parseInt(value, 10);
  } catch (e) {
    throw e;
  }
}

export function coerceToInt(value: string): number {
  // expect: Throw=probable
  return JSON.parse(value) as number;
}

export function validateName(name: string): string {
  // expect: Throw=possible
  return name;
}

export function safeInt(value: string): number {
  // expect: (none — Pure)
  // Throw is caught and not propagated.
  try {
    return JSON.parse(value) as number;
  } catch {
    return 0;
  }
}

export function neverReturns(): never {
  // expect: Throw=definite
  // `never` return type is a definite signal.
  throw new Error("intentional");
}
