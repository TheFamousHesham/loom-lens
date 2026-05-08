"""Functions whose names look like effects but whose bodies are pure.

These exercise the `possible` confidence level: the engine should tag each
with the suggested effect at `possible`, never `probable` or `definite`,
because the body offers no concrete evidence.
"""
from __future__ import annotations


def fetch_default_color() -> str:
    # expect: Net=possible
    # Returns a literal; name matches the `*fetch*` pattern only.
    return "blue"


def save_to_memory(items: list[str], item: str) -> list[str]:
    # expect: IO=possible, Mut=possible
    # Returns a NEW list; original `items` is not mutated. The IO/Mut tags
    # come from the name patterns only.
    return [*items, item]


def update_label(text: str) -> str:
    # expect: Mut=possible
    return text.upper()


def now_str() -> str:
    # expect: Time=possible
    return "now"


def random_word() -> str:
    # expect: Random=probable
    # Method name `.random` on unresolved object — still pure here, but the
    # rule says probable for the call expression style.
    fake = _Fake()
    return fake.random()


class _Fake:
    def random(self) -> str:
        # Pure: the method just returns a constant.
        return "constant"
