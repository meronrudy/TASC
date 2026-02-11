#!/usr/bin/env python3
"""Generate check remediation matrix from spec/tasc/remediation.yaml."""

from __future__ import annotations

import argparse
from pathlib import Path

import yaml


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--checks-file", default="spec/tasc/checks.yaml")
    parser.add_argument("--remediation-file", default="spec/tasc/remediation.yaml")
    parser.add_argument("--output", default="docs/handbook/CHECK_REMEDIATION.md")
    args = parser.parse_args()

    root = Path(args.repo_root).resolve()
    checks = yaml.safe_load((root / args.checks_file).read_text(encoding="utf-8"))["checks"]
    remediation = yaml.safe_load((root / args.remediation_file).read_text(encoding="utf-8"))["checks"]

    lines = [
        "# Check Remediation Matrix",
        "",
        "Generated from `spec/tasc/checks.yaml` and `spec/tasc/remediation.yaml`.",
        "",
        "| Check ID | Category | Remediation |",
        "| --- | --- | --- |",
    ]

    missing: list[str] = []
    for check in checks:
        cid = check["id"]
        category = check.get("category", "-")
        hint = remediation.get(cid)
        if hint is None:
            missing.append(cid)
            hint = "MISSING REMEDIATION"
        lines.append(f"| `{cid}` | `{category}` | {hint} |")

    if missing:
        lines.append("")
        lines.append("## Missing entries")
        for cid in missing:
            lines.append(f"- `{cid}`")

    out_path = (root / args.output).resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if missing:
        print(f"generated {out_path} with missing entries")
        return 1

    print(f"generated {out_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
