#!/usr/bin/env python3
"""Create a deterministic hash manifest for evidence artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as handle:
        while chunk := handle.read(8192):
            h.update(chunk)
    return h.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--input-dir", default="evidence/manifests")
    parser.add_argument("--output", default="evidence/manifests/hashlock.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    input_dir = repo_root / args.input_dir
    output = repo_root / args.output

    files = sorted([p for p in input_dir.glob("**/*") if p.is_file()])
    entries = []
    for f in files:
        stat = f.stat()
        entries.append(
            {
                "path": str(f.relative_to(repo_root)),
                "sha256": sha256_file(f),
                "sizeBytes": stat.st_size,
            }
        )
    entries.sort(key=lambda item: item["path"])
    canonical_bytes = json.dumps(entries, sort_keys=True, separators=(",", ":")).encode("utf-8")
    entries_digest = hashlib.sha256(canonical_bytes).hexdigest()

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(
        json.dumps(
            {
                "hashlockVersion": "0.3.0",
                "entriesDigest": f"sha256:{entries_digest}",
                "entries": entries,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
