// Hash-equality fixtures for the TS parser.
//
// Pairs marked `// duplicate-of: <name>` should produce identical BLAKE3 hashes.
// Pairs marked `// near-duplicate-of: <name>` should NOT, but should be close.

export function addA(x: number, y: number): number {
  return x + y;
}

export function addB(x: number, y: number): number {
  // duplicate-of: addA
  return x + y;
}

export function addC(x: number, y: number): number {
  // near-duplicate-of: addA
  return x - y;
}

export function addRenamed(a: number, b: number): number {
  // near-duplicate-of: addA
  // Identifier names changed; v1 normalization preserves names.
  return a + b;
}

export function addWithLiteral(x: number, y: number): number {
  // near-duplicate-of: addA
  return x + y + 0;
}
