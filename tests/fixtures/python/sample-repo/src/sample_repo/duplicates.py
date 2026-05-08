"""Hash-equality fixtures.

Pairs marked `# duplicate-of: <name>` should produce identical BLAKE3 hashes
under our normalization rules (whitespace/comments stripped; identifiers and
literals preserved). Pairs marked `# near-duplicate-of: <name>` should produce
different hashes but small edit distance over the canonical AST form.
"""
from __future__ import annotations


def add_a(x: int, y: int) -> int:
    return x + y


def add_b(x: int, y: int) -> int:
    # duplicate-of: add_a
    # Identical body; only comments and whitespace differ.
    return x + y


def add_c(x: int, y: int) -> int:
    # near-duplicate-of: add_a
    # Operator changed; should NOT hash-equal.
    return x - y


def add_renamed_params(a: int, b: int) -> int:
    # near-duplicate-of: add_a
    # Identifier names changed; v1 normalization preserves names so this differs.
    return a + b


def add_with_typo(x: int, y: int) -> int:
    # near-duplicate-of: add_a
    # Same shape; literal value introduced.
    return x + y + 0
