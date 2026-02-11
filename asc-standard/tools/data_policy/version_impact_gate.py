#!/usr/bin/env python3
"""Validate data taxonomy/schema compatibility and version-impact declarations."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from pathlib import Path
from typing import Any

import yaml


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return digest


def load_yaml(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must be a YAML object")
    return payload


def load_json(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{path} must be a JSON object")
    return payload


def ensure_version_filename_consistency(path: Path, version: str) -> None:
    match = re.search(r"v(\d+)\.(\d+)", path.name)
    if not match:
        return
    expected_prefix = f"{match.group(1)}.{match.group(2)}"
    if not version.startswith(expected_prefix + ".") and version != expected_prefix:
        raise ValueError(
            f"{path}: version {version} does not match filename prefix v{expected_prefix}"
        )


def collect_check_ids(checks_file: Path) -> set[str]:
    payload = load_yaml(checks_file)
    checks = payload.get("checks", [])
    ids: set[str] = set()
    for check in checks:
        if isinstance(check, dict) and isinstance(check.get("id"), str):
            ids.add(check["id"])
    return ids


def parse_retention_policy(retention_path: Path) -> dict[str, int]:
    text = retention_path.read_text(encoding="utf-8")
    retention_match = re.search(r"minimum retention:\s*(\d+)\s*months", text, re.IGNORECASE)
    windows_match = re.search(
        r"Incident response windows:\s*(\d+)\s*days.*?(\d+)\s*days.*?(\d+)\s*days",
        text,
        re.IGNORECASE | re.DOTALL,
    )
    if retention_match is None or windows_match is None:
        raise ValueError(f"{retention_path} missing retention/windows statements")
    return {
        "retention_months": int(retention_match.group(1)),
        "window_standard": int(windows_match.group(1)),
        "window_widespread": int(windows_match.group(2)),
        "window_fatal": int(windows_match.group(3)),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--impact-map", default="data/version-impact-map.yaml")
    parser.add_argument("--checks-file", default="spec/tasc/checks.yaml")
    parser.add_argument("--output", default="conformance/reports/data-version-impact.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    impact_map_path = (repo_root / args.impact_map).resolve()
    checks_file = (repo_root / args.checks_file).resolve()
    output_path = (repo_root / args.output).resolve()

    impact_map = load_yaml(impact_map_path)
    entries = impact_map.get("artifacts", [])
    if not isinstance(entries, list) or not entries:
        raise SystemExit(f"{impact_map_path} must declare non-empty artifacts list")

    known_checks = collect_check_ids(checks_file)
    if not known_checks:
        raise SystemExit(f"no checks found in {checks_file}")

    errors: list[str] = []
    artifact_reports: list[dict[str, Any]] = []

    for entry in entries:
        if not isinstance(entry, dict):
            errors.append("invalid artifact entry in impact map")
            continue
        artifact_id = str(entry.get("id", ""))
        rel_path = str(entry.get("path", ""))
        version = str(entry.get("version", ""))
        declared_sha = str(entry.get("sha256", ""))
        if not artifact_id or not rel_path or not version or not declared_sha:
            errors.append(f"{artifact_id or rel_path}: missing id/path/version/sha256 in impact map")
            continue

        path = (repo_root / rel_path).resolve()
        if not path.exists():
            errors.append(f"{artifact_id}: missing artifact path {path}")
            continue

        actual_sha = sha256_file(path)
        if actual_sha != declared_sha:
            errors.append(
                f"{artifact_id}: sha256 mismatch (expected {declared_sha}, got {actual_sha})"
            )

        file_version = version
        if path.suffix in {".yaml", ".yml"}:
            payload = load_yaml(path)
            file_version = str(payload.get("version", ""))
            if not file_version:
                errors.append(f"{artifact_id}: YAML payload missing version")
            elif file_version != version:
                errors.append(
                    f"{artifact_id}: version mismatch (impact map {version} vs file {file_version})"
                )

        try:
            ensure_version_filename_consistency(path, version)
        except ValueError as exc:
            errors.append(str(exc))

        impacted_checks = entry.get("impactedChecks", [])
        if not isinstance(impacted_checks, list) or not impacted_checks:
            errors.append(f"{artifact_id}: impactedChecks must be non-empty list")
            impacted_checks = []
        for check_id in impacted_checks:
            if check_id not in known_checks:
                errors.append(f"{artifact_id}: unknown impacted check ID {check_id}")

        artifact_reports.append(
            {
                "id": artifact_id,
                "path": rel_path,
                "version": version,
                "sha256": actual_sha,
                "impactedChecks": impacted_checks,
            }
        )

    # Cross-compatibility checks between data taxonomies and schemas.
    try:
        incident = load_yaml(repo_root / "data/incident-taxonomy/v0.1.yaml")
        incident_schema = load_json(repo_root / "schemas/incident-initial.schema.json")
        incident_levels = [item.get("id") for item in incident.get("levels", []) if isinstance(item, dict)]
        schema_levels = (
            incident_schema.get("properties", {})
            .get("severityLevels", {})
            .get("items", {})
            .get("enum", [])
        )
        if incident_levels != schema_levels:
            errors.append(
                f"incident taxonomy level mismatch: data={incident_levels}, schema={schema_levels}"
            )

        windows = incident.get("reportingWindowsDays", {})
        schema_windows = (
            incident_schema.get("properties", {})
            .get("reportingWindowsDays", {})
            .get("properties", {})
        )
        expected_windows = {
            "standard": schema_windows.get("standard", {}).get("const"),
            "widespread": schema_windows.get("widespread", {}).get("const"),
            "fatal": schema_windows.get("fatal", {}).get("const"),
        }
        actual_windows = {
            "standard": windows.get("standard"),
            "widespread": windows.get("widespread"),
            "fatal": windows.get("fatal"),
        }
        if actual_windows != expected_windows:
            errors.append(
                f"incident reporting windows mismatch: data={actual_windows}, schema={expected_windows}"
            )
    except Exception as exc:
        errors.append(f"incident/schema compatibility check failed: {exc}")

    try:
        logging_data = load_yaml(repo_root / "data/logging-schema/v0.1.yaml")
        logging_schema = load_json(repo_root / "schemas/signed-operational-log.schema.json")
        event_required = (
            logging_schema.get("properties", {})
            .get("events", {})
            .get("items", {})
            .get("required", [])
        )
        allowed_fields = set(event_required + ["signerKeyId", "signature", "root", "schemaVersion", "algorithm"])
        for field in logging_data.get("fields", []):
            if field not in allowed_fields:
                errors.append(f"logging-schema field {field} not present in signed log schema")

        required_alg = (
            logging_schema.get("properties", {})
            .get("algorithm", {})
            .get("const")
        )
        data_alg = logging_data.get("requirements", {}).get("hashAlgorithm")
        if required_alg != data_alg:
            errors.append(
                f"logging hashAlgorithm mismatch: data={data_alg}, schema={required_alg}"
            )
        retention_months = int(
            logging_data.get("requirements", {}).get("retentionMinimumMonths", 0)
        )
        if retention_months < 6:
            errors.append(
                f"logging retentionMinimumMonths must be >= 6, got {retention_months}"
            )
    except Exception as exc:
        errors.append(f"logging/schema compatibility check failed: {exc}")

    try:
        replay_data = load_yaml(repo_root / "data/replay/v0.1.yaml")
        replay_schema = load_json(repo_root / "schemas/replay-recipe.schema.json")
        schema_required = set(replay_schema.get("required", []))
        digest_required = set(replay_data.get("requiredDigests", []))
        expected_digest_required = {
            field
            for field in schema_required
            if field.endswith("Digest")
        }
        if digest_required != expected_digest_required:
            errors.append(
                f"replay requiredDigests mismatch: data={sorted(digest_required)}, schema={sorted(expected_digest_required)}"
            )
        replay_sla_max = int(replay_data.get("reproducibility", {}).get("replaySlaHoursMax", 0))
        if replay_sla_max <= 0:
            errors.append("replay reproducibility.replaySlaHoursMax must be > 0")
    except Exception as exc:
        errors.append(f"replay/schema compatibility check failed: {exc}")

    try:
        retention_policy = parse_retention_policy(repo_root / "data/retention-policy.md")
        if retention_policy["retention_months"] < 6:
            errors.append("retention policy minimum months must be >= 6")
        if (
            retention_policy["window_standard"],
            retention_policy["window_widespread"],
            retention_policy["window_fatal"],
        ) != (15, 2, 10):
            errors.append(
                "retention policy incident windows must be 15/2/10 "
                f"(got {retention_policy['window_standard']}/{retention_policy['window_widespread']}/{retention_policy['window_fatal']})"
            )
    except Exception as exc:
        errors.append(f"retention policy compatibility check failed: {exc}")

    report = {
        "gate": "data-version-impact",
        "impactMap": str(impact_map_path.relative_to(repo_root)),
        "checksFile": str(checks_file.relative_to(repo_root)),
        "artifacts": artifact_reports,
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
