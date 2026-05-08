"""Mutation-effect patterns."""
from __future__ import annotations

from typing import Any

# Module-level mutable: explicit Mut target.
_ERRORS: list[str] = []


def record_error(msg: str) -> None:
    # expect: Mut=definite
    _ERRORS.append(msg)


def update_user(user: dict[str, Any], **changes: Any) -> None:
    # expect: Mut=definite
    user.update(changes)


def add_tag(items: list[str], tag: str) -> None:
    # expect: Mut=definite
    items.append(tag)


def reset_counters() -> None:
    # expect: Mut=definite
    _ERRORS.clear()


class Counter:
    def __init__(self, start: int = 0) -> None:
        self.value = start

    def increment(self) -> None:
        # expect: Mut=definite
        self.value += 1


def add_one(x: int) -> int:
    # expect: Mut=possible
    # Pure body, but name starts with `add_` — possible heuristic only.
    return x + 1
