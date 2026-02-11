#!/usr/bin/env python3
"""Validate basic traceability graph consistency and emit a JSON report."""

from __future__ import annotations

import argparse
import csv
import json
from pathlib import Path


def load_rows(path: Path) -> list[dict[str, str]]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        return list(reader)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--allow-missing-evidence", action="store_true")
    parser.add_argument(
        "--report-path",
        default="evidence/manifests/tracecheck-report.json",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    trace_root = repo_root / "safety-case" / "traceability"

    req_to_spec = load_rows(trace_root / "req_to_spec.csv")
    spec_to_test = load_rows(trace_root / "spec_to_test.csv")
    test_to_evidence = load_rows(trace_root / "test_to_evidence.csv")

    problems: list[str] = []

    test_ids_from_spec = {row["test_id"] for row in spec_to_test}
    test_ids_from_evidence = {row["test_id"] for row in test_to_evidence}

    missing_evidence_map = sorted(test_ids_from_spec - test_ids_from_evidence)
    if missing_evidence_map:
        problems.append(
            f"spec_to_test test IDs missing in test_to_evidence: {missing_evidence_map}"
        )

    missing_files: list[str] = []
    for row in test_to_evidence:
        artifact = repo_root / row["evidence_artifact"]
        if not artifact.exists():
            missing_files.append(row["evidence_artifact"])

    if missing_files and not args.allow_missing_evidence:
        problems.append(f"missing evidence files: {sorted(missing_files)}")

    report = {
        "requirements": len(req_to_spec),
        "spec_to_test_links": len(spec_to_test),
        "test_to_evidence_links": len(test_to_evidence),
        "missing_evidence_mappings": missing_evidence_map,
        "missing_evidence_files": sorted(missing_files),
        "status": "pass" if not problems else "fail",
        "problems": problems,
    }

    report_path = repo_root / args.report_path
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    if problems:
        for problem in problems:
            print(problem)
        return 1

    print(f"tracecheck passed; report: {report_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
