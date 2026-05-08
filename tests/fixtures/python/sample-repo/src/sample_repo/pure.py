"""Pure functions — control case for the inference engine.

None of these should be tagged with any effect.
"""
from __future__ import annotations


def add(a: int, b: int) -> int:
    return a + b


def hypotenuse(a: float, b: float) -> float:
    return (a * a + b * b) ** 0.5


def join_words(words: list[str]) -> str:
    return " ".join(words)


def reverse_pair(pair: tuple[int, int]) -> tuple[int, int]:
    a, b = pair
    return b, a


def upper_first(name: str) -> str:
    if not name:
        return name
    return name[0].upper() + name[1:]


def fibonacci(n: int) -> int:
    if n < 2:
        return n
    a, b = 0, 1
    for _ in range(n - 1):
        a, b = b, a + b
    return b
