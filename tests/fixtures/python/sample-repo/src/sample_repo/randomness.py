"""Random-effect patterns."""
from __future__ import annotations

import random
import secrets
import uuid


def pick_color() -> str:
    # expect: Random=definite
    return random.choice(["red", "green", "blue"])


def make_token() -> str:
    # expect: Random=definite
    return secrets.token_urlsafe(32)


def new_session_id() -> str:
    # expect: Random=definite
    return str(uuid.uuid4())


def shuffle_in_place(items: list[int]) -> None:
    # expect: Random=definite, Mut=definite
    random.shuffle(items)


def fixed_v5(name: str) -> str:
    # expect: (none — Pure)
    # uuid5 is deterministic given the namespace+name; not in the Random rules.
    return str(uuid.uuid5(uuid.NAMESPACE_DNS, name))
