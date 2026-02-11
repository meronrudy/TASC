#!/usr/bin/env python3
"""Compare replay tip hash values from two files."""

from __future__ import annotations

import argparse
from pathlib import Path


def read_hash(path: Path) -> str:
    return path.read_text(encoding="utf-8").strip()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--first", required=True)
    parser.add_argument("--second", required=True)
    args = parser.parse_args()

    first = read_hash(Path(args.first))
    second = read_hash(Path(args.second))

    if first != second:
        print(f"replay hash mismatch: {first} != {second}")
        return 1

    print(f"replay hash match: {first}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
