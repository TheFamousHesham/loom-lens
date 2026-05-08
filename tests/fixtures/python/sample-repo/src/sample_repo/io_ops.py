"""IO-effect patterns."""
from __future__ import annotations

import os
import shutil
import sys
from pathlib import Path


def write_log(path: Path, msg: str) -> None:
    # expect: IO=definite
    with open(path, "a", encoding="utf-8") as f:
        f.write(msg + "\n")


def read_config(path: Path) -> str:
    # expect: IO=probable
    # Reading is IO in our taxonomy, but read mode is `probable` (the rules file says so).
    return path.read_text(encoding="utf-8")


def emit(line: str) -> None:
    # expect: IO=definite
    print(line)


def emit_err(line: str) -> None:
    # expect: IO=definite
    sys.stderr.write(line + "\n")


def remove_temp(path: Path) -> None:
    # expect: IO=definite
    os.remove(path)


def archive(src: Path, dst: Path) -> None:
    # expect: IO=definite
    shutil.copy(src, dst)


def parse_file(path: Path) -> int:
    # expect: IO=possible
    # Pure body — name pattern only. `path` parameter triggers IO=possible heuristic.
    return len(str(path))
