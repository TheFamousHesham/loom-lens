"""Foreign-effect patterns (FFI, subprocess, native modules)."""
from __future__ import annotations

import ctypes
import subprocess


def run_ls() -> str:
    # expect: IO=definite, Foreign=definite
    return subprocess.check_output(["/bin/ls", "/tmp"], text=True)


def libc_strlen(s: bytes) -> int:
    # expect: Foreign=definite
    libc = ctypes.CDLL("libc.so.6")
    libc.strlen.restype = ctypes.c_size_t
    return int(libc.strlen(s))


def safe_native_wrapper(s: str) -> int:
    # expect: Foreign=definite
    return libc_strlen(s.encode())
