"""Throw-effect patterns."""
from __future__ import annotations


class InvalidId(ValueError):
    """Raised when a user id is malformed."""


def raise_if_invalid(user_id: int) -> None:
    # expect: Throw=definite
    if user_id < 0:
        raise InvalidId(f"negative user_id: {user_id}")


def reraise_in_handler(value: str) -> int:
    # expect: Throw=definite, Throw=probable
    try:
        return int(value)
    except ValueError:
        raise


def coerce_to_int(value: str) -> int:
    # expect: Throw=probable
    return int(value)


def assert_positive(n: int) -> int:
    # expect: Throw=probable
    assert n > 0, f"expected positive, got {n}"
    return n


def validate_name(name: str) -> str:
    # expect: Throw=possible
    # Pure body: returns the input. Name pattern only.
    return name


def safe_int(value: str) -> int:
    # expect: (none — Pure)
    # Catches the throw; the function does not propagate it.
    try:
        return int(value)
    except ValueError:
        return 0
