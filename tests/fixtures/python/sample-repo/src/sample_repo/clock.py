"""Time-effect patterns."""
from __future__ import annotations

import time
from datetime import datetime


def now_utc() -> datetime:
    # expect: Time=definite
    return datetime.utcnow()


def monotonic_ns() -> int:
    # expect: Time=definite
    return time.monotonic_ns()


def pause(seconds: float) -> None:
    # expect: Time=definite
    time.sleep(seconds)


def current_time_label() -> str:
    # expect: Time=possible
    # Pure body; name pattern only.
    return "now"
