#!/usr/bin/env python3
"""Prune non-canonical generated artifacts before release packaging."""

from __future__ import annotations

import argparse
import fnmatch
import json
from pathlib import Path
from typing import Any

import yaml


def match_any(path: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(path, pattern) for pattern in patterns)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--manifest-dir", default="evidence/manifests")
    parser.add_argument("--allowlist", default="evidence/canonical-artifacts.yaml")
    parser.add_argument("--output", default="evidence/manifests/canonicalize-report.json")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    manifest_dir = (repo_root / args.manifest_dir).resolve()
    allowlist = yaml.safe_load((repo_root / args.allowlist).read_text(encoding="utf-8")) or {}
    keep = [str(p).strip() for p in allowlist.get("keep", []) if str(p).strip()]

    removed: list[str] = []
    kept: list[str] = []
    for file_path in sorted(manifest_dir.glob("**/*")):
        if not file_path.is_file():
            continue
        rel = str(file_path.relative_to(repo_root))
        if match_any(rel, keep):
            kept.append(rel)
            continue
        removed.append(rel)
        if not args.dry_run:
            file_path.unlink(missing_ok=True)

    payload: dict[str, Any] = {
        "canonicalizeVersion": "0.1.0",
        "dryRun": args.dry_run,
        "manifestDir": str(Path(args.manifest_dir)),
        "allowlist": str(Path(args.allowlist)),
        "kept": kept,
        "removed": removed,
        "result": "PASS",
    }
    output_path = (repo_root / args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"canonicalize removed {len(removed)} files")
    print(f"report: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
