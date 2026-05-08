// Hash-equality fixtures.
//
// `// duplicate-of: <name>` — should hash-equal under v1 normalization.
// `// near-duplicate-of: <name>` — should NOT hash-equal but be close.

pub fn add_a(x: i64, y: i64) -> i64 {
    x + y
}

pub fn add_b(x: i64, y: i64) -> i64 {
    // duplicate-of: add_a
    x + y
}

pub fn add_c(x: i64, y: i64) -> i64 {
    // near-duplicate-of: add_a
    x - y
}

pub fn add_renamed(a: i64, b: i64) -> i64 {
    // near-duplicate-of: add_a
    // Identifier names changed; v1 normalization preserves names.
    a + b
}

pub fn add_with_literal(x: i64, y: i64) -> i64 {
    // near-duplicate-of: add_a
    x + y + 0
}
