#!/usr/bin/env python3
"""Build a release evidence tarball and policy-validated manifest summary."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import tarfile
import time
from pathlib import Path
from typing import Any

import yaml


PROFILES = ["uas-small", "fixed-wing", "hybrid-vtol"]


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_yaml(path: Path) -> dict[str, Any]:
    return yaml.safe_load(path.read_text(encoding="utf-8")) or {}


def sha256_file_prefixed(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    return f"sha256:{digest}"


def digest_of_json(path: Path) -> str:
    payload = load_json(path)
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    digest = hashlib.sha256(canonical).hexdigest()
    return f"sha256:{digest}"


def is_sha_prefixed(value: Any) -> bool:
    if not isinstance(value, str) or not value.startswith("sha256:"):
        return False
    body = value[7:]
    return len(body) == 64 and all(ch in "0123456789abcdef" for ch in body)


def check_age(path: Path, max_age_hours: int, label: str) -> None:
    age_seconds = max(0.0, time.time() - path.stat().st_mtime)
    if age_seconds > max_age_hours * 3600:
        raise SystemExit(
            f"{label} exceeds max age ({int(age_seconds // 3600)}h > {max_age_hours}h): {path}"
        )


def ensure_exists(path: Path, label: str) -> None:
    if not path.exists() or not path.is_file():
        raise SystemExit(f"missing required {label}: {path}")


def validate_conformance(repo_root: Path, required_check_ids: list[str]) -> dict[str, str]:
    reports: dict[str, str] = {}
    for profile in PROFILES:
        report_path = repo_root / f"evidence/manifests/tasc-conformance-{profile}.json"
        ensure_exists(report_path, f"{profile} conformance report")
        report = load_json(report_path)
        if report.get("result") != "PASS":
            raise SystemExit(f"conformance report failed for {profile}: {report_path}")
        present = {entry["id"] for entry in report.get("checks", []) if isinstance(entry, dict)}
        missing = [cid for cid in required_check_ids if cid not in present]
        if missing:
            raise SystemExit(
                f"conformance report missing required checks for {profile}: {', '.join(missing)}"
            )
        reports[profile] = str(report_path.relative_to(repo_root))
    return reports


def validate_lineage(
    *,
    profile: str,
    assurance_pack_path: Path,
    conformance_report_path: Path,
    hashlock_path: Path,
    spec_hash_path: Path,
    required_lineage_fields: list[str],
) -> dict[str, str]:
    assurance = load_json(assurance_pack_path)
    lineage = assurance.get("lineage")
    if not isinstance(lineage, dict):
        raise SystemExit(f"{profile}: assurance pack missing lineage object")

    missing = [field for field in required_lineage_fields if field not in lineage]
    if missing:
        raise SystemExit(f"{profile}: missing lineage fields: {', '.join(missing)}")

    for field in ["specHash", "conformanceReportDigest", "assurancePackDigest", "hashlockDigest"]:
        if not is_sha_prefixed(lineage.get(field)):
            raise SystemExit(f"{profile}: invalid lineage digest field {field}={lineage.get(field)}")

    spec_hash_hex = spec_hash_path.read_text(encoding="utf-8").strip()
    expected_spec_hash = f"sha256:{spec_hash_hex}"
    if lineage["specHash"] != expected_spec_hash:
        raise SystemExit(
            f"{profile}: lineage specHash mismatch "
            f"(expected {expected_spec_hash}, got {lineage['specHash']})"
        )

    expected_conformance_digest = digest_of_json(conformance_report_path)
    if lineage["conformanceReportDigest"] != expected_conformance_digest:
        raise SystemExit(
            f"{profile}: conformanceReportDigest mismatch "
            f"(expected {expected_conformance_digest}, got {lineage['conformanceReportDigest']})"
        )

    expected_hashlock_digest = sha256_file_prefixed(hashlock_path)
    if lineage["hashlockDigest"] != expected_hashlock_digest:
        raise SystemExit(
            f"{profile}: hashlockDigest mismatch "
            f"(expected {expected_hashlock_digest}, got {lineage['hashlockDigest']})"
        )

    transparency = assurance.get("transparencyProofs")
    if not isinstance(transparency, dict):
        raise SystemExit(f"{profile}: transparencyProofs missing in assurance pack")
    for log_id in ["rekor", "mirror"]:
        proof = transparency.get(log_id)
        if not isinstance(proof, dict):
            raise SystemExit(f"{profile}: missing {log_id} transparency proof")
        if proof.get("entryDigest") != lineage["assurancePackDigest"]:
            raise SystemExit(
                f"{profile}: {log_id} entryDigest does not match lineage assurancePackDigest"
            )

    return {
        "profile": profile,
        "assurancePack": str(assurance_pack_path),
        "conformanceReport": str(conformance_report_path),
        "assurancePackDigest": lineage["assurancePackDigest"],
    }


def validate_freshness(
    *,
    repo_root: Path,
    freshness_policy: dict[str, Any],
    hashlock_path: Path,
    release_inputs: list[Path],
) -> None:
    max_age = freshness_policy.get("max_age_hours", {})
    conformance_age = int(max_age.get("conformance_report", 168))
    assurance_age = int(max_age.get("assurance_pack", 168))
    hashlock_age = int(max_age.get("hashlock_manifest", 168))

    for profile in PROFILES:
        conformance = repo_root / f"evidence/manifests/tasc-conformance-{profile}.json"
        assurance = repo_root / f"evidence/manifests/tasc-assurance-pack-{profile}.json"
        check_age(conformance, conformance_age, f"{profile} conformance report")
        check_age(assurance, assurance_age, f"{profile} assurance pack")

    check_age(hashlock_path, hashlock_age, "hashlock manifest")
    for path in release_inputs:
        if not path.exists():
            raise SystemExit(f"release include missing: {path}")


def validate_ordering(
    *,
    spec_hash_path: Path,
    conformance_paths: list[Path],
    assurance_paths: list[Path],
    hashlock_path: Path,
) -> None:
    spec_mtime = spec_hash_path.stat().st_mtime
    min_conformance = min(path.stat().st_mtime for path in conformance_paths)
    min_assurance = min(path.stat().st_mtime for path in assurance_paths)

    if min_conformance < spec_mtime:
        raise SystemExit("ordering violation: conformance report older than spec-hash baseline")
    if min_assurance < spec_mtime:
        raise SystemExit("ordering violation: assurance pack older than spec-hash baseline")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--output", default="evidence/manifests/releasepack.tgz")
    parser.add_argument(
        "--freshness-policy",
        default="policies/provenance/freshness-policy.yaml",
    )
    parser.add_argument(
        "--checks-file",
        default="spec/tasc/checks.yaml",
    )
    parser.add_argument(
        "--include",
        nargs="*",
        default=[
            "evidence/manifests/spec-hash.txt",
            "evidence/manifests/tracecheck-report.json",
            "evidence/manifests/replay-drift.json",
            "evidence/manifests/data-version-impact.json",
            "evidence/manifests/governance-policy-gate.json",
            "evidence/manifests/class-a-gate.json",
            "evidence/manifests/canonicalize-report.json",
            "evidence/manifests/hashlock.json",
            "evidence/manifests/tasc-assurance-pack-uas-small.json",
            "evidence/manifests/tasc-assurance-pack-fixed-wing.json",
            "evidence/manifests/tasc-assurance-pack-hybrid-vtol.json",
            "evidence/manifests/tasc-conformance-uas-small.json",
            "evidence/manifests/tasc-conformance-fixed-wing.json",
            "evidence/manifests/tasc-conformance-hybrid-vtol.json",
        ],
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    out_path = repo_root / args.output
    out_path.parent.mkdir(parents=True, exist_ok=True)

    checks_catalog = load_yaml(repo_root / args.checks_file)
    required_check_ids = [entry["id"] for entry in checks_catalog.get("checks", [])]
    freshness_policy = load_yaml(repo_root / args.freshness_policy)
    required_lineage_fields = list(freshness_policy.get("required_lineage", []))

    include_paths = [repo_root / rel for rel in args.include]
    for path in include_paths:
        ensure_exists(path, "release include")

    spec_hash_path = repo_root / "evidence/manifests/spec-hash.txt"
    hashlock_path = repo_root / "evidence/manifests/hashlock.json"
    ensure_exists(spec_hash_path, "spec hash")
    ensure_exists(hashlock_path, "hashlock manifest")

    conformance_reports = validate_conformance(repo_root, required_check_ids)
    validate_freshness(
        repo_root=repo_root,
        freshness_policy=freshness_policy,
        hashlock_path=hashlock_path,
        release_inputs=include_paths,
    )

    conformance_paths = [repo_root / f"evidence/manifests/tasc-conformance-{p}.json" for p in PROFILES]
    assurance_paths = [repo_root / f"evidence/manifests/tasc-assurance-pack-{p}.json" for p in PROFILES]
    validate_ordering(
        spec_hash_path=spec_hash_path,
        conformance_paths=conformance_paths,
        assurance_paths=assurance_paths,
        hashlock_path=hashlock_path,
    )

    lineage_validation = []
    for profile in PROFILES:
        lineage_validation.append(
            validate_lineage(
                profile=profile,
                assurance_pack_path=repo_root / f"evidence/manifests/tasc-assurance-pack-{profile}.json",
                conformance_report_path=repo_root / f"evidence/manifests/tasc-conformance-{profile}.json",
                hashlock_path=hashlock_path,
                spec_hash_path=spec_hash_path,
                required_lineage_fields=required_lineage_fields,
            )
        )

    included: list[str] = []
    ordered_include = sorted(args.include)
    with out_path.open("wb") as raw_archive:
        with gzip.GzipFile(fileobj=raw_archive, mode="wb", mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode="w") as archive:
                for rel in ordered_include:
                    path = repo_root / rel
                    info = archive.gettarinfo(str(path), arcname=rel)
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.mtime = 0
                    with path.open("rb") as handle:
                        archive.addfile(info, fileobj=handle)
                    included.append(rel)

    manifest = {
        "package": str(Path(args.output)),
        "included": included,
        "conformanceReports": conformance_reports,
        "lineageValidation": lineage_validation,
        "freshnessPolicy": str(Path(args.freshness_policy)),
        "requiredCheckCount": len(required_check_ids),
    }
    release_manifest_path = repo_root / "evidence/manifests/releasepack.json"
    release_manifest_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(f"built {out_path} with {len(included)} files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
