#!/usr/bin/env python3
"""Build offline external-consumer handoff package."""

from __future__ import annotations

import argparse
import gzip
import json
import shutil
import stat
import subprocess
import tarfile
from pathlib import Path


def write_wrapper(path: Path) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    content = """#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: verify-offline.sh <bundle.json> <profile>" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUNDLE="$1"
PROFILE="$2"
BUNDLE_ABS="$(cd "$(dirname "$BUNDLE")" && pwd)/$(basename "$BUNDLE")"
BUNDLE_DIR="$(cd "$(dirname "$BUNDLE_ABS")" && pwd)"

# Deterministic handoff tarballs set mtime=0; refresh local snapshot mtime for
# strict freshness gates that also validate file age.
touch "$PKG_ROOT/policies/attestation/revocation-snapshot.json"
find "$PKG_ROOT/sample" "$PKG_ROOT/policies" -type f \\( -name "*.pem" -o -name "revocation-snapshot.json" \\) -exec touch {} +

cd "$BUNDLE_DIR"
"$PKG_ROOT/bin/tasc-verify" verify \
  --bundle "$BUNDLE_ABS" \
  --profile "$PROFILE" \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror \
  --checks-file "$PKG_ROOT/spec/tasc/checks.yaml" \
  --trusted-checkpoints "$PKG_ROOT/policies/transparency/trusted-log-checkpoints.json" \
  --badge-registry "$PKG_ROOT/policies/badge-registry.json" \
  --attestation-trust-policy "$PKG_ROOT/policies/attestation/trust-policy.json" \
  --trust-roots "$PKG_ROOT/policies/attestation/pki/trust-roots.pem" \
  --revocation-snapshot "$PKG_ROOT/policies/attestation/revocation-snapshot.json" \
  --freshness-policy "$PKG_ROOT/policies/provenance/freshness-policy.yaml" \
  --transparency-policy "$PKG_ROOT/policies/transparency/verification-policy.yaml" \
  --remediation-file "$PKG_ROOT/spec/tasc/remediation.yaml"
"""
    path.write_text(content, encoding="utf-8")
    path.chmod(path.stat().st_mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def copy_tree(src: Path, dst: Path) -> None:
    if dst.exists():
        shutil.rmtree(dst)
    shutil.copytree(src, dst)


def deterministic_tarball(source_dir: Path, output: Path) -> None:
    with output.open("wb") as raw:
        with gzip.GzipFile(fileobj=raw, mode="wb", mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode="w") as archive:
                for file_path in sorted(source_dir.rglob("*")):
                    if file_path.is_dir():
                        continue
                    arcname = str(file_path.relative_to(source_dir))
                    info = archive.gettarinfo(str(file_path), arcname=arcname)
                    info.uid = 0
                    info.gid = 0
                    info.uname = ""
                    info.gname = ""
                    info.mtime = 0
                    with file_path.open("rb") as handle:
                        archive.addfile(info, fileobj=handle)


def first_existing(paths: list[Path]) -> Path:
    for candidate in paths:
        if candidate.exists() and candidate.is_file():
            return candidate
    joined = ", ".join(str(path) for path in paths)
    raise SystemExit(f"none of the expected files exist: {joined}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--version", default="0.2.0")
    parser.add_argument("--sample-profile", default="uas-small")
    parser.add_argument("--output", default="")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    output = (
        Path(args.output).resolve()
        if args.output
        else (repo_root / f"dist/tasc-handoff-{args.version}.tgz").resolve()
    )
    stage = (repo_root / "dist/.handoff-stage").resolve()
    if stage.exists():
        shutil.rmtree(stage)
    stage.mkdir(parents=True, exist_ok=True)

    # Build verifier binary for package.
    subprocess.run(
        ["cargo", "build", "--manifest-path", str(repo_root / "tools/tasc-verify/Cargo.toml")],
        check=True,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    verifier_bin = repo_root / "tools/tasc-verify/target/debug/tasc-verify"
    if not verifier_bin.exists():
        raise SystemExit(f"missing verifier binary: {verifier_bin}")

    (stage / "bin").mkdir(parents=True, exist_ok=True)
    shutil.copy2(verifier_bin, stage / "bin/tasc-verify")
    write_wrapper(stage / "bin/verify-offline.sh")

    copy_tree(repo_root / "schemas", stage / "schemas")
    copy_tree(repo_root / "policies", stage / "policies")
    copy_tree(repo_root / "spec/tasc", stage / "spec/tasc")

    sample_bundle = first_existing(
        [
            repo_root / f"evidence/manifests/tasc-assurance-pack-{args.sample_profile}.json",
            repo_root / f"conformance/fixtures/tasc/tasc-assurance-pack-{args.sample_profile}.json",
        ]
    )
    sample_report = first_existing(
        [
            repo_root / f"evidence/manifests/tasc-conformance-{args.sample_profile}.json",
            repo_root / f"conformance/fixtures/tasc/tasc-conformance-{args.sample_profile}.json",
        ]
    )
    (stage / "sample").mkdir(parents=True, exist_ok=True)
    staged_bundle = stage / f"sample/tasc-assurance-pack-{args.sample_profile}.json"
    staged_report = stage / f"sample/tasc-conformance-{args.sample_profile}.json"
    shutil.copy2(sample_bundle, staged_bundle)
    shutil.copy2(sample_report, staged_report)

    bundle_payload = json.loads(sample_bundle.read_text(encoding="utf-8"))
    artifacts = bundle_payload.get("artifacts", [])
    if isinstance(artifacts, list):
        for entry in artifacts:
            if not isinstance(entry, dict):
                continue
            rel_path = entry.get("path")
            if not isinstance(rel_path, str) or not rel_path.strip():
                continue
            rel_path = rel_path.strip()
            src = (sample_bundle.parent / rel_path).resolve()
            if not src.exists():
                fallback = (repo_root / rel_path).resolve()
                if fallback.exists():
                    src = fallback
            if not src.exists() or src.is_dir():
                continue
            dst = (stage / "sample" / rel_path).resolve()
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)

    for rel_path in [
        bundle_payload.get("signedOperationalLog", {}).get("signerCertificatePath")
        if isinstance(bundle_payload.get("signedOperationalLog"), dict)
        else None,
        bundle_payload.get("signedOperationalLog", {}).get("certificateChainPath")
        if isinstance(bundle_payload.get("signedOperationalLog"), dict)
        else None,
        bundle_payload.get("attestationEvidence", {}).get("signerCertificatePath")
        if isinstance(bundle_payload.get("attestationEvidence"), dict)
        else None,
        bundle_payload.get("attestationEvidence", {}).get("certificateChainPath")
        if isinstance(bundle_payload.get("attestationEvidence"), dict)
        else None,
    ]:
        if not isinstance(rel_path, str) or not rel_path.strip():
            continue
        src = (repo_root / rel_path).resolve()
        if not src.exists() or src.is_dir():
            continue
        dst = (stage / "sample" / rel_path).resolve()
        dst.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dst)

    for key in [
        "shipmentEligibilityCertificate",
        "underwriterConfidencePacket",
        "procurementBidPacket",
        "recyclerIntakePassport",
    ]:
        obj = bundle_payload.get("procurementObjects", {}).get(key) if isinstance(bundle_payload.get("procurementObjects"), dict) else None
        if not isinstance(obj, dict):
            continue
        artifact_ref = obj.get("artifactRef")
        if not isinstance(artifact_ref, dict):
            continue
        rel_path = artifact_ref.get("path")
        if not isinstance(rel_path, str) or not rel_path.strip():
            continue
        src = (sample_bundle.parent / rel_path).resolve()
        if src.exists():
            dst = (stage / "sample" / rel_path).resolve()
            dst.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(src, dst)

    metadata = {
        "handoffVersion": args.version,
        "sampleProfile": args.sample_profile,
        "verifyCommand": "bin/verify-offline.sh sample/tasc-assurance-pack-<profile>.json <profile>",
    }
    (stage / "README.json").write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")

    output.parent.mkdir(parents=True, exist_ok=True)
    deterministic_tarball(stage, output)
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
