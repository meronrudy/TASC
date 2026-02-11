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

"$PKG_ROOT/bin/tasc-verify" verify \
  --bundle "$BUNDLE" \
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

    sample_bundle = repo_root / f"conformance/fixtures/tasc/tasc-assurance-pack-{args.sample_profile}.json"
    sample_report = repo_root / f"conformance/fixtures/tasc/tasc-conformance-{args.sample_profile}.json"
    (stage / "sample").mkdir(parents=True, exist_ok=True)
    shutil.copy2(sample_bundle, stage / f"sample/tasc-assurance-pack-{args.sample_profile}.json")
    shutil.copy2(sample_report, stage / f"sample/tasc-conformance-{args.sample_profile}.json")

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
