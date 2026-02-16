#!/usr/bin/env python3
"""Execute negative vectors and enforce expected failing check IDs + coverage."""

from __future__ import annotations

import argparse
import copy
import json
import shlex
import subprocess
from collections import defaultdict
from pathlib import Path
from typing import Any

import yaml


def pop_path(obj: dict[str, Any], path: str) -> None:
    parts = path.split(".")
    cur: Any = obj
    for key in parts[:-1]:
        if not isinstance(cur, dict) or key not in cur:
            return
        cur = cur[key]
    if isinstance(cur, dict):
        cur.pop(parts[-1], None)


def mutate_for_check(bundle: dict[str, Any], check_id: str, variant: int) -> None:
    v = variant % 3
    if check_id == "CHK_SCHEMA_ASSURANCEPACK":
        for field in ["assurancePackVersion", "profile", "policy"]:
            if v == 0:
                bundle.pop(field, None)
                return
            v -= 1
    elif check_id == "CHK_SCHEMA_EVIDENCEMAP":
        if v == 0:
            bundle.pop("evidenceMap", None)
        elif v == 1:
            pop_path(bundle, "evidenceMap.topology")
        else:
            pop_path(bundle, "evidenceMap.logs")
    elif check_id == "CHK_SCHEMA_CONFORMANCE_REPORT":
        if v == 0:
            bundle.pop("conformanceReport", None)
        elif v == 1:
            pop_path(bundle, "conformanceReport.result")
        else:
            pop_path(bundle, "conformanceReport.checks")
    elif check_id == "CHK_SCHEMA_SIGNED_LOG":
        if v == 0:
            bundle.pop("signedOperationalLog", None)
        elif v == 1:
            pop_path(bundle, "signedOperationalLog.signature")
        else:
            pop_path(bundle, "signedOperationalLog.events")
    elif check_id == "CHK_SCHEMA_REPLAY_RECIPE":
        if v == 0:
            bundle.pop("replayRecipe", None)
        elif v == 1:
            pop_path(bundle, "replayRecipe.seedsDigest")
        else:
            pop_path(bundle, "replayRecipe.buildDigest")
    elif check_id == "CHK_SCHEMA_ATTESTATION":
        if v == 0:
            bundle.pop("attestationEvidence", None)
        elif v == 1:
            pop_path(bundle, "attestationEvidence.level")
        else:
            pop_path(bundle, "attestationEvidence.claims")
    elif check_id == "CHK_SCHEMA_INCIDENT_INITIAL":
        if v == 0:
            bundle.pop("incidentInitialTemplate", None)
        elif v == 1:
            pop_path(bundle, "incidentInitialTemplate.reportingWindowsDays.standard")
        else:
            pop_path(bundle, "incidentInitialTemplate.templateId")
    elif check_id == "CHK_SCHEMA_INCIDENT_PACK":
        if v == 0:
            bundle.pop("incidentPackTemplate", None)
        elif v == 1:
            pop_path(bundle, "incidentPackTemplate.requiredArtifacts")
        else:
            pop_path(bundle, "incidentPackTemplate.fullPackSlaDays")
    elif check_id == "CHK_SCHEMA_TRANSPARENCY_REKOR":
        if v == 0:
            pop_path(bundle, "transparencyProofs.rekor")
        elif v == 1:
            pop_path(bundle, "transparencyProofs.rekor.logId")
        else:
            pop_path(bundle, "transparencyProofs.rekor.checkpoint")
    elif check_id == "CHK_SCHEMA_TRANSPARENCY_MIRROR":
        if v == 0:
            pop_path(bundle, "transparencyProofs.mirror")
        elif v == 1:
            pop_path(bundle, "transparencyProofs.mirror.logId")
        else:
            pop_path(bundle, "transparencyProofs.mirror.checkpoint")
    elif check_id == "CHK_SCHEMA_BADGE_ENTRY":
        if v == 0:
            bundle.pop("badgeEntry", None)
        elif v == 1:
            pop_path(bundle, "badgeEntry.badgeId")
        else:
            pop_path(bundle, "badgeEntry.badgeType")
    elif check_id == "CHK_SCHEMA_PROCUREMENT_OBJECTS":
        if v == 0:
            bundle.pop("procurementObjects", None)
        elif v == 1:
            pop_path(bundle, "releaseContext.lifecycleStage")
        else:
            pop_path(bundle, "procurementObjects.shipmentEligibilityCertificate.objectType")
    elif check_id == "CHK_PROCUREMENT_OBJECT_REQUIRED_SET":
        if v == 0:
            pop_path(bundle, "procurementObjects.procurementBidPacket")
        elif v == 1:
            pop_path(bundle, "procurementObjects.underwriterConfidencePacket")
        else:
            bundle.setdefault("releaseContext", {})["lifecycleStage"] = "decommission"
            pop_path(bundle, "procurementObjects.recyclerIntakePassport")
    elif check_id == "CHK_PROCUREMENT_OBJECT_ARTIFACT_PARITY":
        obj = (
            bundle.get("procurementObjects", {}).get("procurementBidPacket")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict):
            artifact_ref = obj.get("artifactRef")
            if isinstance(artifact_ref, dict):
                if v == 0:
                    artifact_ref["path"] = str(artifact_ref.get("path", "")) + ".missing"
                elif v == 1:
                    artifact_ref["sha256"] = "sha256:" + "f" * 64
                else:
                    obj["objectName"] = "Tampered Procurement Bid Packet"
    elif check_id == "CHK_PROCUREMENT_OBJECT_INPUT_HASH":
        obj = (
            bundle.get("procurementObjects", {}).get("shipmentEligibilityCertificate")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict):
            if v == 0:
                obj["inputHash"] = "sha256:" + "0" * 64
            elif v == 1:
                if isinstance(obj.get("inputs"), dict):
                    obj["inputs"]["profile"] = "tampered-profile"
            else:
                if isinstance(obj.get("inputs"), dict):
                    obj["inputs"]["requiredTransparency"] = ["mirror"]
    elif check_id == "CHK_PROCUREMENT_OBJECT_BUNDLE_BINDING":
        obj = (
            bundle.get("procurementObjects", {}).get("underwriterConfidencePacket")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict):
            obj["bundleDigest"] = "sha256:" + ("1" if v == 0 else "2" if v == 1 else "3") * 64
    elif check_id == "CHK_PROCUREMENT_OBJECT_SIGNATURE":
        obj = (
            bundle.get("procurementObjects", {}).get("procurementBidPacket")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict):
            envelope = obj.get("signatureEnvelope")
            signer = obj.get("signer")
            if isinstance(envelope, dict) and isinstance(signer, dict):
                if v == 0:
                    envelope["signature"] = "broken-signature"
                elif v == 1:
                    envelope["payloadDigest"] = "sha256:" + "4" * 64
                else:
                    signer["signatureAlgorithm"] = "ecdsa-sha256"
    elif check_id == "CHK_PROCUREMENT_OBJECT_TRUST_FLOOR":
        obj = (
            bundle.get("procurementObjects", {}).get("underwriterConfidencePacket")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict) and isinstance(obj.get("signer"), dict):
            if v == 0:
                obj["signer"]["trustAnchorLevel"] = "TA1"
            elif v == 1:
                obj["signer"]["trustAnchorLevel"] = "TA0"
            else:
                obj["signer"]["keySource"] = "FILE"
    elif check_id == "CHK_PROCUREMENT_OBJECT_VALIDITY_WINDOW":
        obj = (
            bundle.get("procurementObjects", {}).get("shipmentEligibilityCertificate")
            if isinstance(bundle.get("procurementObjects"), dict)
            else None
        )
        if isinstance(obj, dict) and isinstance(obj.get("validity"), dict):
            if v == 0:
                obj["validity"]["notAfterUtc"] = "2020-01-01T00:00:00Z"
            elif v == 1:
                obj["validity"]["notBeforeUtc"] = "2099-01-01T00:00:00Z"
            else:
                obj["validity"]["notBeforeUtc"] = "2030-01-01T00:00:00Z"
                obj["validity"]["notAfterUtc"] = "2029-01-01T00:00:00Z"
    elif check_id == "CHK_RECYCLER_INTAKE_CONDITIONAL":
        bundle.setdefault("releaseContext", {})["lifecycleStage"] = "decommission"
        if v == 0:
            pop_path(bundle, "procurementObjects.recyclerIntakePassport")
        elif v == 1:
            pop_path(bundle, "procurementObjects")
        else:
            pop_path(bundle, "releaseContext.lifecycleStage")
    elif check_id == "CHK_PROFILE_MATCH":
        bundle["profile"] = f"wrong-profile-{v}"
    elif check_id == "CHK_POLICY_MATCH":
        bundle["policy"] = f"wrong-policy-{v}"
    elif check_id == "CHK_TOPOLOGY_INTERLOCK_MEDIATION":
        for edge in bundle["evidenceMap"]["topology"]["edges"]:
            if edge.get("signal") == "actuator_command":
                edge["from"] = "n_ai_planner" if v == 0 else "n_safety_kernel"
                break
    elif check_id == "CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY":
        for edge in bundle["evidenceMap"]["topology"]["edges"]:
            if edge.get("to") == "n_interlock_gate":
                edge["from"] = ["n_ai_planner", "n_actuator_bus", "n_ai_planner"][v]
                break
    elif check_id == "CHK_TOPOLOGY_ASSERTION_REFS":
        assertions = bundle["evidenceMap"]["topology"]["conformanceAssertions"]
        if assertions:
            if v == 0:
                assertions[0]["proofRef"] = "bad-proof-ref"
            elif v == 1:
                assertions[0]["status"] = "unknown"
            else:
                assertions[:] = [a for a in assertions if a.get("id") != "A001"]
    elif check_id == "CHK_ARTIFACT_HASH_INTEGRITY":
        artifacts = bundle.get("artifacts", [])
        if artifacts:
            target = min(v, len(artifacts) - 1)
            artifacts[target]["sha256"] = "sha256:" + ("0" if v == 0 else "1" if v == 1 else "2") * 64
    elif check_id == "CHK_LOG_HASH_CHAIN":
        events = bundle["signedOperationalLog"]["events"]
        if len(events) > 1:
            events[1]["prevHash"] = "sha256:" + str(v + 1) * 64
    elif check_id == "CHK_LOG_SIGNATURE":
        bundle["signedOperationalLog"]["signature"] = f"invalid-signature-{v}"
    elif check_id == "CHK_RETENTION_EU_MINIMUM":
        bundle["evidenceMap"]["logs"]["retentionPolicy"]["minimumMonths"] = [0, 1, 5][v]
    elif check_id == "CHK_REPLAY_COMPLETENESS":
        fields = ["seedsDigest", "configDigest", "buildDigest"]
        bundle["replayRecipe"].pop(fields[v], None)
    elif check_id == "CHK_REPLAY_OPERATIONAL_PARITY":
        artifacts = bundle.get("artifacts", [])
        for artifact in artifacts:
            path = str(artifact.get("path", ""))
            if "replay-from-log-" in path and path.endswith(".json"):
                if v == 0:
                    artifact["path"] = f"{path}.missing"
                elif v == 1:
                    artifact["path"] = "evidence/manifests/hashlock.json"
                else:
                    artifact["path"] = "evidence/manifests/spec-hash.txt"
                return
        bundle.setdefault("artifacts", []).append({"path": "missing-replay-report.json", "sha256": "sha256:" + "3" * 64})
    elif check_id == "CHK_ATTESTATION_TA2":
        if v == 0:
            bundle["attestationEvidence"]["level"] = "TA1"
        elif v == 1:
            bundle["attestationEvidence"]["claims"]["hsmBacked"] = False
        else:
            bundle["attestationEvidence"]["keySource"] = "FILE"
    elif check_id == "CHK_INCIDENT_WINDOWS_EU":
        if v == 0:
            bundle["incidentInitialTemplate"]["reportingWindowsDays"]["standard"] = 14
        elif v == 1:
            bundle["incidentInitialTemplate"]["reportingWindowsDays"]["widespread"] = 3
        else:
            bundle["incidentInitialTemplate"]["reportingWindowsDays"]["fatal"] = 9
    elif check_id == "CHK_TRANSPARENCY_REKOR_PROOF":
        if v == 0:
            bundle["transparencyProofs"]["rekor"]["signature"] = "broken"
        elif v == 1:
            bundle["transparencyProofs"]["rekor"]["entryDigest"] = "sha256:" + "f" * 64
        else:
            bundle["transparencyProofs"]["rekor"]["source"] = "local"
    elif check_id == "CHK_TRANSPARENCY_MIRROR_PROOF":
        if v == 0:
            bundle["transparencyProofs"]["mirror"]["signature"] = "broken"
        elif v == 1:
            bundle["transparencyProofs"]["mirror"]["entryDigest"] = "sha256:" + "e" * 64
        else:
            bundle["transparencyProofs"]["mirror"]["source"] = "local"
    elif check_id == "CHK_BADGE_ACTIVE_NOT_REVOKED":
        bundle["badgeEntry"]["status"] = "revoked" if v == 0 else "suspended"
    else:
        raise ValueError(f"unsupported check id: {check_id}")


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
    return subprocess.run(cmd, text=True, capture_output=True)


def load_vectors(repo_root: Path, vectors_index_path: Path) -> list[dict[str, Any]]:
    payload = yaml.safe_load(vectors_index_path.read_text(encoding="utf-8"))
    vectors = payload.get("vectors", [])
    for vector in vectors:
        check_id = vector["checkId"]
        vector_path = repo_root / vector["path"]
        vector_payload = json.loads(vector_path.read_text(encoding="utf-8"))
        if vector_payload.get("targetCheckId") != check_id:
            raise SystemExit(f"vector targetCheckId mismatch: {vector_path}")
    return vectors


def load_required_checks(checks_catalog_path: Path) -> list[str]:
    payload = yaml.safe_load(checks_catalog_path.read_text(encoding="utf-8"))
    checks = payload.get("checks", [])
    required: list[str] = []
    for check in checks:
        if check.get("required", True):
            check_id = check.get("id")
            if isinstance(check_id, str) and check_id:
                required.append(check_id)
    return required


def run_profile(
    repo_root: Path,
    vectors: list[dict[str, Any]],
    bundle_path: Path,
    profile: str,
    policy: str,
    require_ta: str,
    require_transparency: str,
    verifier_bin: str | None,
    verifier_cmd: str,
    report_dir: Path,
    min_vectors_per_check: int,
) -> tuple[bool, list[dict[str, Any]], dict[str, int]]:
    base_bundle = json.loads(bundle_path.read_text(encoding="utf-8"))
    profile_ok = True
    results: list[dict[str, Any]] = []
    coverage: dict[str, int] = defaultdict(int)

    for vector in vectors:
        check_id = vector["checkId"]
        for variant in range(min_vectors_per_check):
            mutated = copy.deepcopy(base_bundle)
            mutate_for_check(mutated, check_id, variant)

            temp_bundle = bundle_path.parent / f".neg-{profile}-{check_id}-v{variant}.json"
            temp_report = report_dir / f"tasc-neg-{profile}-{check_id}-v{variant}.json"
            temp_bundle.write_text(json.dumps(mutated, indent=2) + "\n", encoding="utf-8")

            try:
                proc = run_verify(
                    verifier_bin=verifier_bin,
                    verifier_cmd=verifier_cmd,
                    bundle_path=temp_bundle,
                    profile=profile,
                    policy=policy,
                    require_ta=require_ta,
                    require_transparency=require_transparency,
                    output_path=temp_report,
                )
            finally:
                temp_bundle.unlink(missing_ok=True)

            if not temp_report.exists():
                profile_ok = False
                results.append(
                    {
                        "checkId": check_id,
                        "variant": variant,
                        "vector": vector["path"],
                        "result": "FAIL",
                        "reason": "missing verifier report",
                        "returnCode": proc.returncode,
                        "stderr": proc.stderr,
                    }
                )
                continue

            verify_report = json.loads(temp_report.read_text(encoding="utf-8"))
            failed_checks = verify_report.get("failedChecks", [])
            expected_seen = check_id in failed_checks
            verifier_failed = proc.returncode != 0
            ok = verifier_failed and expected_seen
            profile_ok = profile_ok and ok
            if ok:
                coverage[check_id] += 1

            results.append(
                {
                    "checkId": check_id,
                    "variant": variant,
                    "vector": vector["path"],
                    "expected": check_id,
                    "seenInFailedChecks": expected_seen,
                    "verifierReturnCode": proc.returncode,
                    "verifierResult": verify_report.get("result"),
                    "failedChecks": failed_checks,
                    "result": "PASS" if ok else "FAIL",
                }
            )
            temp_report.unlink(missing_ok=True)

    for vector in vectors:
        check_id = vector["checkId"]
        if coverage.get(check_id, 0) < min_vectors_per_check:
            profile_ok = False
            results.append(
                {
                    "checkId": check_id,
                    "variant": "coverage",
                    "vector": vector["path"],
                    "result": "FAIL",
                    "reason": f"coverage below minimum {min_vectors_per_check} (got {coverage.get(check_id, 0)})",
                }
            )

    return profile_ok, results, dict(coverage)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--vectors-index", default="conformance/vectors/tasc-negative-index.yaml")
    parser.add_argument("--checks-catalog", default="spec/tasc/checks.yaml")
    parser.add_argument(
        "--bundle-template",
        default="conformance/fixtures/tasc/tasc-assurance-pack-{profile}.json",
    )
    parser.add_argument("--profiles", default="uas-small")
    parser.add_argument("--policy", default="eu-north-star")
    parser.add_argument("--require-ta", default="TA2")
    parser.add_argument("--require-transparency", default="rekor,mirror")
    parser.add_argument("--verifier-bin", default=None)
    parser.add_argument(
        "--verifier-cmd",
        default="cargo run --manifest-path tools/tasc-verify/Cargo.toml --",
    )
    parser.add_argument("--min-vectors-per-check", type=int, default=3)
    parser.add_argument("--output", default="conformance/reports/tasc-negative-vectors.json")
    parser.add_argument("--coverage-output", default="conformance/reports/tasc-negative-coverage.json")
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    vectors = load_vectors(repo_root, (repo_root / args.vectors_index).resolve())
    required_checks = load_required_checks((repo_root / args.checks_catalog).resolve())
    vector_check_ids = [vector["checkId"] for vector in vectors]
    missing_checks = sorted(set(required_checks) - set(vector_check_ids))
    extra_checks = sorted(set(vector_check_ids) - set(required_checks))
    if missing_checks:
        raise SystemExit(
            "negative vectors missing required checks: " + ", ".join(missing_checks)
        )
    if extra_checks:
        raise SystemExit(
            "negative vectors include unknown checks: " + ", ".join(extra_checks)
        )
    report_dir = (repo_root / "conformance/reports").resolve()
    report_dir.mkdir(parents=True, exist_ok=True)

    profiles = [part.strip() for part in args.profiles.split(",") if part.strip()]
    all_profiles: list[dict[str, Any]] = []
    coverage_profiles: list[dict[str, Any]] = []
    overall_ok = True

    for profile in profiles:
        bundle_path = (repo_root / args.bundle_template.format(profile=profile)).resolve()
        if not bundle_path.exists():
            raise SystemExit(f"bundle missing for profile {profile}: {bundle_path}")

        profile_ok, results, coverage = run_profile(
            repo_root=repo_root,
            vectors=vectors,
            bundle_path=bundle_path,
            profile=profile,
            policy=args.policy,
            require_ta=args.require_ta,
            require_transparency=args.require_transparency,
            verifier_bin=args.verifier_bin,
            verifier_cmd=args.verifier_cmd,
            report_dir=report_dir,
            min_vectors_per_check=args.min_vectors_per_check,
        )
        overall_ok = overall_ok and profile_ok
        all_profiles.append(
            {
                "profile": profile,
                "bundle": str(bundle_path.relative_to(repo_root)),
                "result": "PASS" if profile_ok else "FAIL",
                "results": results,
            }
        )
        coverage_profiles.append(
            {
                "profile": profile,
                "minimumVectorsPerCheck": args.min_vectors_per_check,
                "coverageByCheckId": coverage,
                "result": "PASS"
                if all(coverage.get(v["checkId"], 0) >= args.min_vectors_per_check for v in vectors)
                else "FAIL",
            }
        )

    output = {
        "negativeVectorsVersion": "0.3.0",
        "policy": args.policy,
        "requireTa": args.require_ta,
        "requireTransparency": [part.strip() for part in args.require_transparency.split(",") if part.strip()],
        "profiles": all_profiles,
        "result": "PASS" if overall_ok else "FAIL",
    }
    coverage_output = {
        "negativeCoverageVersion": "0.1.0",
        "minimumVectorsPerCheck": args.min_vectors_per_check,
        "requiredChecks": required_checks,
        "profiles": coverage_profiles,
        "result": "PASS"
        if all(profile["result"] == "PASS" for profile in coverage_profiles)
        else "FAIL",
    }

    out_path = (repo_root / args.output).resolve()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(json.dumps(output, indent=2) + "\n", encoding="utf-8")
    coverage_path = (repo_root / args.coverage_output).resolve()
    coverage_path.parent.mkdir(parents=True, exist_ok=True)
    coverage_path.write_text(json.dumps(coverage_output, indent=2) + "\n", encoding="utf-8")

    for profile_payload in all_profiles:
        print(f"{profile_payload['profile']}: {profile_payload['result']}")
    print(f"overall: {output['result']}")
    print(f"report: {out_path}")
    print(f"coverage: {coverage_path}")

    return 0 if overall_ok and coverage_output["result"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
