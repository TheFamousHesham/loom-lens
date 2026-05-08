// Time-effect patterns.

export function nowMs(): number {
  // expect: Time=definite
  return Date.now();
}

export function timestamp(): Date {
  // expect: Time=definite
  return new Date();
}

export function highRes(): number {
  // expect: Time=definite
  return performance.now();
}

export function dateFromIso(iso: string): Date {
  // expect: (none — Pure)
  // `new Date(arg)` with an argument is pure construction.
  return new Date(iso);
}

export function nowLabel(): string {
  // expect: Time=possible
  return "now";
}
