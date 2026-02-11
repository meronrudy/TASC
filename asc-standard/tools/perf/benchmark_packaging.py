#!/usr/bin/env python3
"""Benchmark assurancepack (with live publish) and releasepack packaging runtimes."""

from __future__ import annotations

import argparse
import json
import subprocess
import time
from pathlib import Path


def p95(samples: list[float]) -> float:
    if not samples:
        return 0.0
    ordered = sorted(samples)
    idx = int(round(0.95 * (len(ordered) - 1)))
    return ordered[idx]


def run_cmd(command: list[str], cwd: Path, label: str) -> float:
    start = time.perf_counter()
    proc = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    elapsed = time.perf_counter() - start
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"{label} failed: {detail}")
    return elapsed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--profiles", default="uas-small,fixed-wing,hybrid-vtol")
    parser.add_argument("--iterations", type=int, default=2)
    parser.add_argument("--signer-mode", default="pkcs11")
    parser.add_argument("--pkcs11-profile", default="policies/attestation/pkcs11-profile.yaml")
    parser.add_argument("--pkcs11-module", default="")
    parser.add_argument("--pkcs11-token-label", default="tasc-soft-token")
    parser.add_argument("--pkcs11-key-label", default="tasc-ta2-key")
    parser.add_argument("--pkcs11-cert-label", default="tasc-ta2-cert")
    parser.add_argument("--pkcs11-pin-env", default="TASC_PKCS11_PIN")
    parser.add_argument("--pkcs11-mechanism", default="SHA256-RSA-PKCS")
    parser.add_argument("--rekor-url", default="https://rekor.sigstore.dev")
    parser.add_argument("--mirror-url", default="http://127.0.0.1:17777")
    parser.add_argument("--output", default="conformance/reports/perf-packaging.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    profiles = [p.strip() for p in args.profiles.split(",") if p.strip()]
    assurance_profile_reports = []
    for profile in profiles:
        samples = []
        for iteration in range(args.iterations):
            out = repo_root / f"conformance/reports/perf-assurancepack-{profile}-{iteration}.json"
            archive = repo_root / f"conformance/reports/perf-assurancepack-{profile}-{iteration}.tgz"
            cmd = [
                "python3",
                "tools/assurancepack/assurancepack.py",
                "--repo-root",
                ".",
                "--profile",
                profile,
                "--output",
                str(out),
                "--archive",
                str(archive),
                "--badge-id",
                f"badge-{profile}-active",
                "--signer-mode",
                args.signer_mode,
                "--pkcs11-profile",
                args.pkcs11_profile,
                "--pkcs11-token-label",
                args.pkcs11_token_label,
                "--pkcs11-key-label",
                args.pkcs11_key_label,
                "--pkcs11-cert-label",
                args.pkcs11_cert_label,
                "--pkcs11-pin-env",
                args.pkcs11_pin_env,
                "--pkcs11-mechanism",
                args.pkcs11_mechanism,
                "--publish-live",
                "--rekor-url",
                args.rekor_url,
                "--mirror-url",
                args.mirror_url,
                "--transparency-retries",
                "4",
                "--transparency-backoff-seconds",
                "2",
            ]
            if args.pkcs11_module:
                cmd.extend(["--pkcs11-module", args.pkcs11_module])
            samples.append(run_cmd(cmd, repo_root, f"assurancepack {profile}"))
        assurance_profile_reports.append(
            {"profile": profile, "samplesSeconds": samples, "p95Seconds": p95(samples)}
        )

    releasepack_samples = []
    for _ in range(args.iterations):
        releasepack_samples.append(
            run_cmd(
                ["python3", "tools/releasepack/releasepack.py", "--repo-root", "."],
                repo_root,
                "releasepack",
            )
        )

    payload = {
        "benchmark": "packaging",
        "iterations": args.iterations,
        "assurancepackProfiles": assurance_profile_reports,
        "releasepack": {"samplesSeconds": releasepack_samples, "p95Seconds": p95(releasepack_samples)},
    }
    output = (repo_root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
