#!/usr/bin/env python3
"""Benchmark replay-from-log runtime across profiles."""

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


def run_replay(repo_root: Path, profile: str, trace: Path, output: Path) -> float:
    cmd = [
        "cargo",
        "run",
        "--manifest-path",
        "reference/kernel/Cargo.toml",
        "-p",
        "asc-conformance-kernel",
        "--bin",
        "replay-from-log",
        "--",
        "--repo-root",
        str(repo_root),
        "--profile",
        profile,
        "--trace",
        str(trace),
        "--output",
        str(output),
    ]
    start = time.perf_counter()
    proc = subprocess.run(cmd, cwd=repo_root, capture_output=True, text=True, check=False)
    elapsed = time.perf_counter() - start
    if proc.returncode != 0:
        detail = (proc.stderr or proc.stdout or "").strip()
        raise RuntimeError(f"replay-from-log failed for {profile}: {detail}")
    return elapsed


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--profiles", default="uas-small,fixed-wing,hybrid-vtol")
    parser.add_argument(
        "--trace-template",
        default="conformance/fixtures/tasc/runtime-e2e-{profile}/operational-trace-{profile}-ga-smoke.json",
    )
    parser.add_argument("--iterations", type=int, default=3)
    parser.add_argument("--output", default="conformance/reports/perf-replay.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    profiles = [p.strip() for p in args.profiles.split(",") if p.strip()]
    report_profiles = []
    for profile in profiles:
        trace = (repo_root / args.trace_template.format(profile=profile)).resolve()
        if not trace.exists():
            raise SystemExit(f"missing replay trace: {trace}")
        samples: list[float] = []
        for iteration in range(args.iterations):
            out = repo_root / f"conformance/reports/perf-replay-{profile}-{iteration}.json"
            samples.append(run_replay(repo_root=repo_root, profile=profile, trace=trace, output=out))
        report_profiles.append(
            {
                "profile": profile,
                "samplesSeconds": samples,
                "p95Seconds": p95(samples),
                "eventCount": 50000,
            }
        )

    payload = {"benchmark": "replay-from-log", "iterations": args.iterations, "profiles": report_profiles}
    output = (repo_root / args.output).resolve()
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"wrote {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
