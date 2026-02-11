#!/usr/bin/env python3
"""Class A governance gate: detect safety-critical diffs and require DR/RA/CI artifacts."""

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import subprocess
from pathlib import Path, PurePosixPath
from typing import Any

import yaml


def run(command: list[str], cwd: Path) -> list[str]:
    proc = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    if proc.returncode != 0:
        raise RuntimeError((proc.stderr or proc.stdout or "").strip())
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def changed_files(repo_root: Path, base_ref: str | None) -> list[str]:
    if base_ref:
        try:
            return run(["git", "diff", "--name-only", f"{base_ref}...HEAD"], repo_root)
        except Exception:
            pass
    try:
        return run(["git", "diff", "--name-only", "HEAD~1..HEAD"], repo_root)
    except Exception:
        return run(["git", "ls-files"], repo_root)


def matches_any(path: str, globs: list[str]) -> bool:
    pp = PurePosixPath(path)
    for pattern in globs:
        if pp.match(pattern) or fnmatch.fnmatch(path, pattern):
            return True
    return False


def first_matches(paths: list[str], pattern: str) -> list[str]:
    return [path for path in paths if matches_any(path, [pattern])]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument(
        "--scope",
        default="governance/class-a-scope.yaml",
    )
    parser.add_argument("--base-ref", default=None)
    parser.add_argument(
        "--output",
        default="conformance/reports/class-a-gate.json",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    scope_path = (repo_root / args.scope).resolve()
    scope = yaml.safe_load(scope_path.read_text(encoding="utf-8")) or {}
    class_a_globs = [str(value).strip() for value in scope.get("class_a_globs", []) if str(value).strip()]
    required = scope.get("required_records", {}) or {}
    dr_glob = str(required.get("decision_record_glob", "")).strip()
    ra_glob = str(required.get("risk_acceptance_glob", "")).strip()
    ci_glob = str(required.get("change_impact_glob", "")).strip()

    base_ref = args.base_ref
    if not base_ref:
        base_env = os.getenv("GITHUB_BASE_REF", "").strip()
        if base_env:
            base_ref = f"origin/{base_env}"

    changed = changed_files(repo_root, base_ref)
    class_a_changed = [path for path in changed if matches_any(path, class_a_globs)]
    class_a = bool(class_a_changed)

    decision_records = first_matches(changed, dr_glob) if dr_glob else []
    risk_acceptance = first_matches(changed, ra_glob) if ra_glob else []
    change_impact = first_matches(changed, ci_glob) if ci_glob else []

    passed = True
    reasons: list[str] = []
    if class_a:
        if not decision_records:
            passed = False
            reasons.append(f"missing changed decision record matching {dr_glob}")
        if not risk_acceptance:
            passed = False
            reasons.append(f"missing changed risk acceptance artifact matching {ra_glob}")
        if not change_impact:
            passed = False
            reasons.append(f"missing changed change-impact artifact matching {ci_glob}")

    payload: dict[str, Any] = {
        "class": "A" if class_a else "B",
        "result": "PASS" if passed else "FAIL",
        "baseRef": base_ref or "",
        "changedFiles": changed,
        "classAScopeMatches": class_a_changed,
        "requiredArtifacts": {
            "decisionRecords": decision_records,
            "riskAcceptance": risk_acceptance,
            "changeImpact": change_impact,
        },
        "reasons": reasons,
    }
    output_path = (repo_root / args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")

    if not passed:
        for reason in reasons:
            print(f"FAIL: {reason}")
        return 1
    print(f"class-a-gate: {payload['result']} ({payload['class']})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
