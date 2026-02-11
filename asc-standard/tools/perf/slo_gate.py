#!/usr/bin/env python3
"""Evaluate performance benchmark outputs against SLO policy."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

import yaml


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--policy", default="policies/perf/slo.yaml")
    parser.add_argument("--verify-report", default="conformance/reports/perf-verify.json")
    parser.add_argument("--replay-report", default="conformance/reports/perf-replay.json")
    parser.add_argument("--packaging-report", default="conformance/reports/perf-packaging.json")
    parser.add_argument("--output", default="conformance/reports/perf-slo.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    policy = yaml.safe_load((repo_root / args.policy).read_text(encoding="utf-8"))
    verify = json.loads((repo_root / args.verify_report).read_text(encoding="utf-8"))
    replay = json.loads((repo_root / args.replay_report).read_text(encoding="utf-8"))
    packaging = json.loads((repo_root / args.packaging_report).read_text(encoding="utf-8"))

    slo = policy.get("slo", {})
    failures: list[str] = []
    checks: list[dict[str, Any]] = []

    verify_limit = float(slo.get("verify_p95_seconds", 20))
    for item in verify.get("profiles", []):
        value = float(item.get("p95Seconds", 0))
        ok = value <= verify_limit
        checks.append({"metric": f"verify:{item.get('profile')}", "p95Seconds": value, "limitSeconds": verify_limit, "result": "PASS" if ok else "FAIL"})
        if not ok:
            failures.append(f"verify p95 for {item.get('profile')} exceeded {verify_limit}s (got {value}s)")

    replay_limit = float(slo.get("replay_from_log_p95_seconds", 60))
    for item in replay.get("profiles", []):
        value = float(item.get("p95Seconds", 0))
        ok = value <= replay_limit
        checks.append({"metric": f"replay:{item.get('profile')}", "p95Seconds": value, "limitSeconds": replay_limit, "result": "PASS" if ok else "FAIL"})
        if not ok:
            failures.append(f"replay p95 for {item.get('profile')} exceeded {replay_limit}s (got {value}s)")

    assurance_limit = float(slo.get("assurancepack_publish_p95_seconds", 30))
    for item in packaging.get("assurancepackProfiles", []):
        value = float(item.get("p95Seconds", 0))
        ok = value <= assurance_limit
        checks.append({"metric": f"assurancepack:{item.get('profile')}", "p95Seconds": value, "limitSeconds": assurance_limit, "result": "PASS" if ok else "FAIL"})
        if not ok:
            failures.append(f"assurancepack p95 for {item.get('profile')} exceeded {assurance_limit}s (got {value}s)")

    releasepack_limit = float(slo.get("releasepack_p95_seconds", 30))
    releasepack_value = float(packaging.get("releasepack", {}).get("p95Seconds", 0))
    releasepack_ok = releasepack_value <= releasepack_limit
    checks.append({"metric": "releasepack", "p95Seconds": releasepack_value, "limitSeconds": releasepack_limit, "result": "PASS" if releasepack_ok else "FAIL"})
    if not releasepack_ok:
        failures.append(f"releasepack p95 exceeded {releasepack_limit}s (got {releasepack_value}s)")

    payload = {"policyVersion": policy.get("version", ""), "checks": checks, "result": "PASS" if not failures else "FAIL", "failures": failures}
    output = (repo_root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
