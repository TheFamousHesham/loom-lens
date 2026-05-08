"""Async-effect patterns."""
from __future__ import annotations

import asyncio


async def fetch_all(urls: list[str]) -> list[bytes]:
    # expect: Async=definite
    return await asyncio.gather(*[_fetch_one(u) for u in urls])


async def _fetch_one(url: str) -> bytes:
    # expect: Async=definite
    await asyncio.sleep(0)
    return url.encode()


def run_blocking() -> None:
    # expect: Async=definite
    asyncio.run(fetch_all([]))


def worker_loop() -> None:
    # expect: Async=possible
    # Pure body; name suggests concurrency.
    return None


async def trio_style_helper() -> int:
    # expect: Async=definite
    return 42
