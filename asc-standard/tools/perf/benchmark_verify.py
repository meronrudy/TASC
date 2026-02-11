#!/usr/bin/env python3
"""Benchmark tasc-verify runtime across profiles."""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import time
from pathlib import Path


def p95(samples: list[float]) -> float:
    if not samples:
        return 0.0
    ordered = sorted(samples)
    idx = int(round(0.95 * (len(ordered) - 1)))
    return ordered[idx]


def run_verify(
    repo_root: Path,
    verifier_cmd: str,
    bundle: Path,
    profile: str,
    policy: str,
    require_ta: str,
    require_transparency: str,
) -> float:
    cmd = shlex.split(verifier_cmd) + [
        "verify",
        "--bundle",
        str(bundle),
        "--profile",
        profile,
        "--policy",
        policy,
        "--require-ta",
        require_ta,
        "--require-transparency",
        require_transparency,
    ]
    start = time.perf_counter()
    proc = subprocess.run(cmd, cwd=repo_root, capture_output=True, text=True, check=False)
    elapsed = time.perf_counter() - start
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"verify failed for {profile}: {detail}")
    return elapsed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--profiles", default="uas-small,fixed-wing,hybrid-vtol")
    parser.add_argument(
        "--bundle-template",
        default="conformance/fixtures/tasc/tasc-assurance-pack-{profile}.json",
    )
    parser.add_argument("--iterations", type=int, default=3)
    parser.add_argument("--policy", default="eu-north-star")
    parser.add_argument("--require-ta", default="TA2")
    parser.add_argument("--require-transparency", default="rekor,mirror")
    parser.add_argument(
        "--verifier-cmd",
        default="cargo run --manifest-path tools/tasc-verify/Cargo.toml --",
    )
    parser.add_argument("--output", default="conformance/reports/perf-verify.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    profiles = [p.strip() for p in args.profiles.split(",") if p.strip()]
    report_profiles = []
    for profile in profiles:
        bundle = (repo_root / args.bundle_template.format(profile=profile)).resolve()
        if not bundle.exists():
            raise SystemExit(f"missing bundle: {bundle}")
        samples = [
            run_verify(
                repo_root=repo_root,
                verifier_cmd=args.verifier_cmd,
                bundle=bundle,
                profile=profile,
                policy=args.policy,
                require_ta=args.require_ta,
                require_transparency=args.require_transparency,
            )
            for _ in range(args.iterations)
        ]
        report_profiles.append(
            {
                "profile": profile,
                "samplesSeconds": samples,
                "p95Seconds": p95(samples),
            }
        )

    payload = {
        "benchmark": "verify",
        "iterations": args.iterations,
        "profiles": report_profiles,
    }
    output = (repo_root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
