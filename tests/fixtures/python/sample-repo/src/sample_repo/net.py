"""Network-effect patterns."""
from __future__ import annotations

import socket
from typing import Any

import requests
import httpx

from .errors import raise_if_invalid


def fetch_user(user_id: int) -> dict[str, Any]:
    # expect: Net=definite, Throw=probable
    raise_if_invalid(user_id)
    response = requests.get(f"https://api.example/users/{user_id}", timeout=10)
    response.raise_for_status()
    return response.json()


def post_event(payload: dict[str, Any]) -> None:
    # expect: Net=definite
    requests.post("https://api.example/events", json=payload, timeout=5)


async def stream_events() -> list[str]:
    # expect: Net=definite, Async=definite
    async with httpx.AsyncClient() as client:
        resp = await client.get("https://api.example/events/stream")
        return resp.text.splitlines()


def open_socket(host: str, port: int) -> socket.socket:
    # expect: Net=definite
    s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    s.connect((host, port))
    return s


def fetch_things(client: Any, ids: list[int]) -> list[Any]:
    # expect: Net=probable
    # `client` is unresolved; the .get call on a name suggesting Client-ness is probable.
    return [client.get(f"/thing/{i}") for i in ids]


def make_request_helper(url: str) -> str:
    # expect: Net=possible
    # Function name matches *request* heuristic but the body is pure (returns the URL).
    return url.rstrip("/")
