#!/usr/bin/env python3
"""Governance policy gate for release-readiness controls."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import yaml


REQUIRED_DOC_MARKERS: dict[str, list[str]] = {
    "governance/CHARTER.md": [
        "Authority Model",
        "Decision Cadence",
        "Escalation Matrix",
        "Required Artifacts",
    ],
    "governance/CHANGE_CONTROL.md": [
        "Class A",
        "Class B",
        "Class C",
        "Review and Approval SLA",
    ],
    "governance/VERSIONING_POLICY.md": [
        "Semantic Versioning Rules",
        "Compatibility Requirements",
        "Class A Freeze Window",
    ],
    "governance/DEPRECATION_POLICY.md": [
        "Support Window",
        "Removal Criteria",
    ],
    "governance/RISK_ACCEPTANCE_POLICY.md": [
        "Exception SLA",
        "Approval Authority",
        "Expiry and Renewal",
    ],
    "governance/WORKING_GROUPS.md": [
        "Spec WG",
        "Kernel WG",
        "Assurance WG",
        "Security WG",
        "Release WG",
    ],
    "governance/COMPLIANCE_POLICY.md": [
        "Mandatory Gates",
        "Class A Requirements",
    ],
    "governance/BRANCH_PROTECTION.md": [
        "Required Status Checks",
        "Application Method",
    ],
}


def load_yaml(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must be a YAML object")
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--output", default="conformance/reports/governance-policy-gate.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    output_path = (repo_root / args.output).resolve()
    errors: list[str] = []
    checked: list[dict[str, Any]] = []

    for rel_path, markers in REQUIRED_DOC_MARKERS.items():
        path = (repo_root / rel_path).resolve()
        if not path.exists():
            errors.append(f"missing governance doc: {rel_path}")
            continue
        text = path.read_text(encoding="utf-8")
        missing = [marker for marker in markers if marker not in text]
        if missing:
            errors.append(f"{rel_path} missing markers: {', '.join(missing)}")
        checked.append({"path": rel_path, "markers": markers, "missing": missing})

    for template_rel in [
        "governance/DECISION_RECORDS/template.md",
        "governance/MEETING_NOTES/template.md",
        "governance/proposals/template.md",
    ]:
        path = repo_root / template_rel
        if not path.exists():
            errors.append(f"missing governance template: {template_rel}")

    status_path = repo_root / "governance/branch-protection.status.yaml"
    if not status_path.exists():
        errors.append("missing governance/branch-protection.status.yaml")
    else:
        status = load_yaml(status_path)
        branch = status.get("branch")
        if branch != "main":
            errors.append("branch-protection.status branch must be 'main'")
        required = status.get("requiredChecks", [])
        if required != ["ci", "conformance", "kernel-ci", "release"]:
            errors.append(
                "branch-protection.status requiredChecks must be [ci, conformance, kernel-ci, release]"
            )
        applied = status.get("applied")
        if not isinstance(applied, bool):
            errors.append("branch-protection.status applied must be boolean")
        elif not applied:
            errors.append("branch-protection.status applied must be true for GA readiness")
        applied_at = status.get("appliedAtUtc")
        if applied and not isinstance(applied_at, str):
            errors.append("branch-protection.status appliedAtUtc must be an RFC3339 string when applied=true")
        status_value = status.get("status")
        if applied and status_value != "applied":
            errors.append("branch-protection.status status must be 'applied' when applied=true")

    report = {
        "gate": "governance-policy",
        "checkedDocs": checked,
        "errors": errors,
        "result": "FAIL" if errors else "PASS",
    }
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(f"result: {report['result']}")
    print(f"report: {output_path}")
    if errors:
        for error in errors:
            print(f"- {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
