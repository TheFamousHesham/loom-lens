// Pure functions — control case for the inference engine.
// None of these should be tagged with any effect.

export function add(a: number, b: number): number {
  return a + b;
}

export function hypotenuse(a: number, b: number): number {
  return Math.sqrt(a * a + b * b);
}

export function joinWords(words: ReadonlyArray<string>): string {
  return words.join(" ");
}

export function reversePair<A, B>(pair: readonly [A, B]): readonly [B, A] {
  const [a, b] = pair;
  return [b, a];
}

export function upperFirst(name: string): string {
  if (!name) return name;
  return name[0]!.toUpperCase() + name.slice(1);
}

export function fibonacci(n: number): number {
  if (n < 2) return n;
  let a = 0,
    b = 1;
  for (let i = 0; i < n - 1; i++) {
    [a, b] = [b, a + b];
  }
  return b;
}
