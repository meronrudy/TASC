#!/usr/bin/env python3
"""Deterministic mutation/fuzz harness for topology/log/signature edge cases."""

from __future__ import annotations

import argparse
import copy
import json
import random
import shlex
import subprocess
from pathlib import Path
from typing import Any


def apply_mutation(bundle: dict[str, Any], seed: int) -> str:
    rnd = random.Random(seed)
    mode = seed % 3
    if mode == 0:
        # Topology authority mutation
        edges = bundle["evidenceMap"]["topology"]["edges"]
        for edge in edges:
            if edge.get("to") == "n_interlock_gate":
                edge["from"] = rnd.choice(["n_ai_planner", "n_actuator_bus"])
                return "topology-authority"
        edges[0]["from"] = "n_ai_planner"
        return "topology-authority"
    if mode == 1:
        # Log chain mutation
        events = bundle["signedOperationalLog"]["events"]
        if events:
            idx = min(len(events) - 1, max(0, rnd.randint(0, len(events) - 1)))
            events[idx]["hash"] = "sha256:" + "".join(rnd.choice("0123456789abcdef") for _ in range(64))
        return "log-hash-chain"
    # Transparency signature mutation
    proof = rnd.choice(["rekor", "mirror"])
    bundle["transparencyProofs"][proof]["signature"] = f"invalid-sig-{seed}"
    return f"transparency-{proof}-signature"


def run_verify(
    verifier_bin: str | None,
    verifier_cmd: str,
    bundle_path: Path,
    profile: str,
    policy: str,
    require_ta: str,
    require_transparency: str,
    output_path: Path,
) -> subprocess.CompletedProcess[str]:
    if verifier_bin:
        cmd = [
            verifier_bin,
            "verify",
            "--bundle",
            str(bundle_path),
            "--profile",
            profile,
            "--policy",
            policy,
            "--require-ta",
            require_ta,
            "--require-transparency",
            require_transparency,
            "--output",
            str(output_path),
        ]
    else:
        cmd = shlex.split(verifier_cmd) + [
            "verify",
            "--bundle",
            str(bundle_path),
            "--profile",
            profile,
            "--policy",
            policy,
            "--require-ta",
            require_ta,
            "--require-transparency",
            require_transparency,
            "--output",
            str(output_path),
        ]
    return subprocess.run(cmd, capture_output=True, text=True, check=False)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument(
        "--bundle-template",
        default="conformance/fixtures/tasc/tasc-assurance-pack-{profile}.json",
    )
    parser.add_argument("--profiles", default="uas-small,fixed-wing,hybrid-vtol")
    parser.add_argument("--seeds", default="11,23,37,41,53,67")
    parser.add_argument("--policy", default="eu-north-star")
    parser.add_argument("--require-ta", default="TA2")
    parser.add_argument("--require-transparency", default="rekor,mirror")
    parser.add_argument("--verifier-bin", default=None)
    parser.add_argument(
        "--verifier-cmd",
        default="cargo run --manifest-path tools/tasc-verify/Cargo.toml --",
    )
    parser.add_argument("--output", default="conformance/reports/tasc-mutation-fuzz.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    profiles = [item.strip() for item in args.profiles.split(",") if item.strip()]
    seeds = [int(item.strip()) for item in args.seeds.split(",") if item.strip()]
    report_dir = repo_root / "conformance/reports"
    report_dir.mkdir(parents=True, exist_ok=True)

    overall_ok = True
    profile_reports: list[dict[str, Any]] = []
    for profile in profiles:
        bundle_path = (repo_root / args.bundle_template.format(profile=profile)).resolve()
        if not bundle_path.exists():
            raise SystemExit(f"bundle not found for profile {profile}: {bundle_path}")
        base_bundle = json.loads(bundle_path.read_text(encoding="utf-8"))

        runs: list[dict[str, Any]] = []
        profile_ok = True
        for seed in seeds:
            mutated = copy.deepcopy(base_bundle)
            mutation = apply_mutation(mutated, seed)
            fuzz_bundle = bundle_path.parent / f".fuzz-{profile}-{seed}.json"
            fuzz_report = report_dir / f"tasc-fuzz-{profile}-{seed}.json"
            fuzz_bundle.write_text(json.dumps(mutated, indent=2) + "\n", encoding="utf-8")
            try:
                proc = run_verify(
                    verifier_bin=args.verifier_bin,
                    verifier_cmd=args.verifier_cmd,
                    bundle_path=fuzz_bundle,
                    profile=profile,
                    policy=args.policy,
                    require_ta=args.require_ta,
                    require_transparency=args.require_transparency,
                    output_path=fuzz_report,
                )
            finally:
                fuzz_bundle.unlink(missing_ok=True)
            expected_fail = proc.returncode != 0
            profile_ok = profile_ok and expected_fail
            failed_checks: list[str] = []
            if fuzz_report.exists():
                failed_checks = json.loads(fuzz_report.read_text(encoding="utf-8")).get(
                    "failedChecks", []
                )
                fuzz_report.unlink(missing_ok=True)
            runs.append(
                {
                    "seed": seed,
                    "mutation": mutation,
                    "returnCode": proc.returncode,
                    "failedChecks": failed_checks,
                    "result": "PASS" if expected_fail else "FAIL",
                }
            )

        overall_ok = overall_ok and profile_ok
        profile_reports.append(
            {"profile": profile, "result": "PASS" if profile_ok else "FAIL", "runs": runs}
        )

    payload = {
        "mutationFuzzVersion": "0.1.0",
        "policy": args.policy,
        "requireTa": args.require_ta,
        "requireTransparency": [v for v in args.require_transparency.split(",") if v],
        "profiles": profile_reports,
        "result": "PASS" if overall_ok else "FAIL",
    }
    output_path = (repo_root / args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    print(f"mutation-fuzz: {payload['result']}")
    print(f"report: {output_path}")
    return 0 if overall_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
