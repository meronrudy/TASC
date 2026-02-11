#!/usr/bin/env python3
"""Offline smoke validator for TASC fixture bundles.

This script mirrors the check IDs in spec/tasc/checks.yaml and validates all
fixture bundles without compiling or running Rust binaries.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import subprocess
import tempfile
import time
from datetime import datetime, timezone
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import yaml


@dataclass
class CheckResult:
    id: str
    result: str
    message: str


@dataclass
class ProfileReport:
    profile: str
    bundle: str
    result: str
    checks: list[CheckResult]
    failedChecks: list[str]


@dataclass
class Context:
    repo_root: Path
    bundle_path: Path
    bundle: dict[str, Any]
    profile: str
    policy: str
    require_ta: str
    require_transparency: list[str]
    trusted_map: dict[str, dict[str, Any]]
    badge_registry: dict[str, Any]
    attestation_policy: dict[str, Any]
    trust_roots: Path
    revocation_snapshot: Path
    freshness_policy: dict[str, Any]
    transparency_policy: dict[str, Any]


def sha256_hex_bytes(data: bytes) -> str:
    h = hashlib.sha256()
    h.update(data)
    return h.hexdigest()


def sha256_hex_str(text: str) -> str:
    return sha256_hex_bytes(text.encode("utf-8"))


def sha256_file_prefixed(path: Path) -> str:
    return f"sha256:{sha256_hex_bytes(path.read_bytes())}"


def is_sha_prefixed(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    if not value.startswith("sha256:"):
        return False
    body = value[7:]
    return len(body) == 64 and all(c in "0123456789abcdef" for c in body)


def parse_csv_list(value: str) -> list[str]:
    return [part.strip() for part in value.split(",") if part.strip()]


def resolve_artifact_path(path: str, bundle_dir: Path) -> Path:
    candidate = Path(path)
    if candidate.is_absolute() or candidate.exists():
        return candidate
    joined = bundle_dir / path
    if joined.exists():
        return joined
    for ancestor in [bundle_dir, *bundle_dir.parents]:
        via_ancestor = ancestor / path
        if via_ancestor.exists():
            return via_ancestor
    return candidate


def resolve_policy_path(path: Path, bundle_dir: Path) -> Path:
    if path.is_absolute():
        return path
    if path.exists():
        return path
    return bundle_dir / path


def check_file_age_hours(path: Path, max_age_hours: int, label: str) -> tuple[bool, str]:
    if not path.exists():
        return False, f"{label} file missing: {path}"
    try:
        age_seconds = max(0.0, time.time() - path.stat().st_mtime)
    except OSError as exc:
        return False, f"failed to stat {path}: {exc}"
    max_age_seconds = max_age_hours * 3600
    if age_seconds > max_age_seconds:
        return (
            False,
            f"{label} exceeded max age: {int(age_seconds // 3600)}h > {max_age_hours}h for {path}",
        )
    return True, f"{label} freshness ok"


def run_checked(command: list[str], label: str) -> tuple[bool, str]:
    try:
        proc = subprocess.run(command, capture_output=True, text=True, check=False)
    except OSError as exc:
        return False, f"{label} failed to execute: {exc}"
    if proc.returncode != 0:
        stderr = (proc.stderr or "").strip()
        stdout = (proc.stdout or "").strip()
        detail = stderr or stdout or f"exit {proc.returncode}"
        return False, f"{label} failed: {detail}"
    return True, proc.stdout


def verify_certificate_chain(
    signer_cert: Path, chain_path: Path, trust_roots: Path
) -> tuple[bool, str]:
    return run_checked(
        [
            "openssl",
            "verify",
            "-CAfile",
            str(trust_roots),
            "-untrusted",
            str(chain_path),
            str(signer_cert),
        ],
        "openssl verify",
    )


def verify_signature_base64(payload: str, signature_b64: str, signer_cert: Path) -> tuple[bool, str]:
    try:
        signature_raw = base64.b64decode(signature_b64, validate=True)
    except Exception as exc:
        return False, f"invalid base64 signature encoding: {exc}"

    with tempfile.NamedTemporaryFile(delete=False) as payload_file, tempfile.NamedTemporaryFile(
        delete=False
    ) as sig_file, tempfile.NamedTemporaryFile(delete=False) as pubkey_file:
        payload_path = Path(payload_file.name)
        sig_path = Path(sig_file.name)
        pubkey_path = Path(pubkey_file.name)
        payload_file.write(payload.encode("utf-8"))
        sig_file.write(signature_raw)

    ok, out = run_checked(
        ["openssl", "x509", "-in", str(signer_cert), "-pubkey", "-noout"],
        "openssl x509 -pubkey",
    )
    if not ok:
        for path in [payload_path, sig_path, pubkey_path]:
            path.unlink(missing_ok=True)
        return False, out

    pubkey_path.write_text(out, encoding="utf-8")
    ok, verify_out = run_checked(
        [
            "openssl",
            "dgst",
            "-sha256",
            "-verify",
            str(pubkey_path),
            "-signature",
            str(sig_path),
            str(payload_path),
        ],
        "openssl dgst -verify",
    )

    for path in [payload_path, sig_path, pubkey_path]:
        path.unlink(missing_ok=True)
    if not ok:
        return False, verify_out
    return True, "signature validated"


def parse_utc(input_value: str) -> tuple[bool, datetime | str]:
    raw = input_value.strip()
    if not raw:
        return False, "timestamp is empty"
    try:
        if raw.endswith("Z"):
            parsed = datetime.fromisoformat(raw.replace("Z", "+00:00"))
        else:
            parsed = datetime.fromisoformat(raw)
    except ValueError as exc:
        return False, f"invalid RFC3339 timestamp {raw}: {exc}"
    if parsed.tzinfo is None:
        return False, f"timestamp must be timezone-aware: {raw}"
    return True, parsed.astimezone(timezone.utc)


def certificate_serial_hex(cert_path: Path) -> tuple[bool, str]:
    ok, out = run_checked(
        ["openssl", "x509", "-in", str(cert_path), "-serial", "-noout"],
        "openssl x509 -serial",
    )
    if not ok:
        return False, out
    raw = out.strip()
    if "=" not in raw:
        return False, f"unexpected serial output: {raw}"
    return True, raw.split("=", 1)[1].strip()


def certificate_text(cert_path: Path) -> tuple[bool, str]:
    return run_checked(
        ["openssl", "x509", "-in", str(cert_path), "-text", "-noout"],
        "openssl x509 -text",
    )


def pointer(data: Any, path: str) -> Any:
    if path == "":
        return data
    if not path.startswith("/"):
        return None

    current = data
    for raw_token in path.lstrip("/").split("/"):
        token = raw_token.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict):
            if token not in current:
                return None
            current = current[token]
        elif isinstance(current, list):
            if not token.isdigit():
                return None
            idx = int(token)
            if idx < 0 or idx >= len(current):
                return None
            current = current[idx]
        else:
            return None
    return current


def check_required_paths(bundle: dict[str, Any], paths: list[str], label: str) -> tuple[bool, str]:
    missing = [path for path in paths if pointer(bundle, path) is None]
    if not missing:
        return True, f"{label} present"
    return False, f"missing {label} fields: {', '.join(missing)}"


def check_bundle_value(bundle: dict[str, Any], path: str, expected: str, label: str) -> tuple[bool, str]:
    value = pointer(bundle, path)
    if value is None:
        return False, f"{label} missing at {path}"
    if value != expected:
        return False, f"{label} mismatch: expected {expected}, got {value}"
    return True, f"{label} matches {expected}"


def topology_nodes(bundle: dict[str, Any]) -> list[dict[str, Any]]:
    nodes = pointer(bundle, "/evidenceMap/topology/nodes")
    if not isinstance(nodes, list):
        raise ValueError("missing or invalid topology nodes")
    return nodes


def topology_edges(bundle: dict[str, Any]) -> list[dict[str, Any]]:
    edges = pointer(bundle, "/evidenceMap/topology/edges")
    if not isinstance(edges, list):
        raise ValueError("missing or invalid topology edges")
    return edges


def check_interlock_mediation(bundle: dict[str, Any]) -> tuple[bool, str]:
    try:
        nodes = topology_nodes(bundle)
        edges = topology_edges(bundle)
    except ValueError as exc:
        return False, str(exc)

    node_types = {n.get("id"): n.get("type") for n in nodes}
    actuator_edges = 0
    for edge in edges:
        if node_types.get(edge.get("to")) == "actuator_integrator":
            actuator_edges += 1
            if node_types.get(edge.get("from")) != "interlock_gate":
                return (
                    False,
                    f"actuator edge {edge.get('from')} -> {edge.get('to')} not mediated by interlock_gate",
                )

    if actuator_edges == 0:
        return False, "no actuator_integrator edges present"
    return True, "all actuator paths mediated by interlock gate"


def check_authority_exclusivity(bundle: dict[str, Any]) -> tuple[bool, str]:
    try:
        nodes = topology_nodes(bundle)
        edges = topology_edges(bundle)
    except ValueError as exc:
        return False, str(exc)

    node_types = {n.get("id"): n.get("type") for n in nodes}
    authority_edges = 0
    for edge in edges:
        if node_types.get(edge.get("to")) == "interlock_gate":
            authority_edges += 1
            if node_types.get(edge.get("from")) != "safety_kernel":
                return (
                    False,
                    f"non-kernel source {edge.get('from')} targets interlock gate {edge.get('to')}",
                )

    if authority_edges == 0:
        return False, "no interlock authority edges present"
    return True, "interlock authority exclusive to safety kernel"


def check_assertion_refs(bundle: dict[str, Any]) -> tuple[bool, str]:
    assertions = pointer(bundle, "/evidenceMap/topology/conformanceAssertions")
    if not isinstance(assertions, list):
        return False, "missing conformanceAssertions"

    for required in ["A001", "A002"]:
        found = next((a for a in assertions if a.get("id") == required), None)
        if found is None:
            return False, f"required assertion {required} missing"
        if found.get("status") != "proven":
            return False, f"assertion {required} not proven"
        if not is_sha_prefixed(found.get("proofRef")):
            return False, f"assertion {required} proofRef invalid"

    return True, "required assertion refs are proven and hash-addressed"


def check_artifact_hashes(ctx: Context) -> tuple[bool, str]:
    artifacts = ctx.bundle.get("artifacts")
    if not isinstance(artifacts, list) or not artifacts:
        return False, "artifacts list missing or empty"

    for artifact in artifacts:
        if not isinstance(artifact, dict):
            return False, "invalid artifact entry"
        rel = artifact.get("path")
        expected = artifact.get("sha256")
        if not isinstance(rel, str) or not isinstance(expected, str):
            return False, "artifact entry missing path or sha256"
        path = resolve_artifact_path(rel, ctx.bundle_path.parent)
        if not path.exists():
            return False, f"missing artifact file {path}"
        got = sha256_file_prefixed(path)
        if got != expected:
            return False, f"artifact digest mismatch for {rel} (expected {expected}, got {got})"

    return True, "artifact hashes validated"


def hash_log_event(event: dict[str, Any]) -> str:
    payload = (
        f"{event.get('seq')}|{event.get('prevHash')}|{event.get('payloadDigest')}|"
        f"{event.get('eventType')}|{event.get('tickMs')}"
    )
    return f"sha256:{sha256_hex_str(payload)}"


def transparency_signature_payload(proof: dict[str, Any]) -> str:
    return (
        f"{proof.get('logId')}|{proof.get('entryUuid')}|{proof.get('logIndex')}|{proof.get('treeSize')}|"
        f"{proof.get('rootHash')}|{proof.get('leafHash')}|{proof.get('checkpointHash')}|{proof.get('entryDigest')}|"
        f"{proof.get('inclusionProofHash')}|{proof.get('consistencyProofHash')}|"
        f"{proof.get('integratedTimeUtc')}"
    )


def compute_attestation_evidence_digest(bundle: dict[str, Any], att: dict[str, Any]) -> str:
    profile = str(bundle.get("profile", ""))
    log_root = str(pointer(bundle, "/signedOperationalLog/root") or "")
    software_state_digest = str(att.get("claims", {}).get("softwareStateDigest", ""))
    payload = (
        f"profile={profile}|logRoot={log_root}|softwareStateDigest={software_state_digest}"
    )
    return f"sha256:{sha256_hex_str(payload)}"


def check_log_hash_chain(bundle: dict[str, Any]) -> tuple[bool, str]:
    log = bundle.get("signedOperationalLog")
    if not isinstance(log, dict):
        return False, "missing signedOperationalLog"
    events = log.get("events")
    if not isinstance(events, list) or not events:
        return False, "log events are empty"

    last_hash = ""
    for idx, event in enumerate(events):
        if not isinstance(event, dict):
            return False, f"event {idx} has invalid type"
        prev_hash = event.get("prevHash")
        if idx == 0:
            if prev_hash != "":
                return False, "first event prevHash must be empty"
        elif prev_hash != last_hash:
            return False, f"event {idx} prevHash does not match prior hash"

        expected = hash_log_event(event)
        if event.get("hash") != expected:
            return (
                False,
                f"event {idx} hash mismatch (expected {expected}, got {event.get('hash')})",
            )
        last_hash = expected

    if log.get("root") != last_hash:
        return False, f"log root mismatch (expected {last_hash}, got {log.get('root')})"

    return True, "log hash chain verified"


def check_log_signature(ctx: Context) -> tuple[bool, str]:
    log = ctx.bundle.get("signedOperationalLog")
    if not isinstance(log, dict):
        return False, "missing signedOperationalLog"

    schema_version = str(log.get("schemaVersion", ""))
    root = log.get("root")
    kid = log.get("signerKeyId")
    sig = log.get("signature")
    if not isinstance(root, str) or not isinstance(kid, str) or not isinstance(sig, str):
        return False, "signedOperationalLog missing root/signerKeyId/signature"

    if schema_version != "0.2":
        return False, f"unsupported signedOperationalLog schemaVersion {schema_version}"

    if log.get("signatureAlgorithm") != "rsa-sha256" or log.get("signatureEncoding") != "base64":
        return False, "0.2 log signature must use rsa-sha256/base64"
    if not str(log.get("signingTimeUtc", "")).strip():
        return False, "signingTimeUtc is required for schemaVersion 0.2"
    if not kid.strip():
        return False, "signerKeyId is required for schemaVersion 0.2"

    max_age = int(ctx.freshness_policy.get("max_age_hours", {}).get("assurance_pack", 168))
    signer_cert_rel = str(log.get("signerCertificatePath", ""))
    signer_cert_path = resolve_artifact_path(signer_cert_rel, ctx.bundle_path.parent)
    ok, message = check_file_age_hours(signer_cert_path, max_age, "signer certificate freshness")
    if not ok:
        return False, message

    chain_path = resolve_artifact_path(
        str(log.get("certificateChainPath", "")), ctx.bundle_path.parent
    )
    trust_roots = resolve_policy_path(ctx.trust_roots, ctx.bundle_path.parent)
    if not signer_cert_path.exists() or not chain_path.exists() or not trust_roots.exists():
        return False, "missing cert/chain/trust roots for log signature"

    expected_trust_digest = sha256_file_prefixed(trust_roots)
    if expected_trust_digest != log.get("trustRootsDigest"):
        return (
            False,
            f"trustRootsDigest mismatch (expected {expected_trust_digest}, got {log.get('trustRootsDigest')})",
        )

    ok, message = verify_certificate_chain(signer_cert_path, chain_path, trust_roots)
    if not ok:
        return False, f"certificate chain verification failed: {message}"
    ok, message = verify_signature_base64(root, sig, signer_cert_path)
    if not ok:
        return False, f"signature verification failed: {message}"
    if ctx.require_ta == "TA2" and log.get("keySource") != "PKCS11":
        return (
            False,
            f"TA2 log signature requires keySource=PKCS11, got {log.get('keySource')}",
        )

    key_source = str(log.get("keySource", "")).strip() or "unknown"
    return True, f"log signature validated with {key_source} key source"


def check_retention_minimum(bundle: dict[str, Any], minimum_months: int) -> tuple[bool, str]:
    value = pointer(bundle, "/evidenceMap/logs/retentionPolicy/minimumMonths")
    if not isinstance(value, int):
        return False, "missing retention minimum"
    if value < minimum_months:
        return False, f"retention minimum {value} months below required {minimum_months}"
    return True, f"retention minimum satisfied ({value} months)"


def check_replay(ctx: Context) -> tuple[bool, str]:
    replay = ctx.bundle.get("replayRecipe")
    if not isinstance(replay, dict):
        return False, "missing replayRecipe"

    for key in [
        "seedsDigest",
        "buildDigest",
        "configDigest",
        "environmentDigest",
        "containerDigest",
    ]:
        if not is_sha_prefixed(replay.get(key)):
            return False, f"{key} is not valid sha256 digest"

    if replay.get("replaySlaHours", 0) <= 0:
        return False, "replaySlaHours must be > 0"

    if not str(replay.get("reproducibilityGuarantee", "")).strip():
        return False, "reproducibilityGuarantee missing"

    if replay.get("recipeVersion") != "0.1":
        return False, f"unsupported replay recipe version {replay.get('recipeVersion')}"

    if replay.get("profile") != ctx.profile:
        return False, f"replay profile {replay.get('profile')} != expected {ctx.profile}"

    return True, "replay completeness checks passed"


def check_replay_operational_parity(ctx: Context) -> tuple[bool, str]:
    artifacts = ctx.bundle.get("artifacts")
    if not isinstance(artifacts, list):
        return False, "artifacts list missing"
    candidate = None
    for artifact in artifacts:
        if not isinstance(artifact, dict):
            continue
        path = str(artifact.get("path", ""))
        if "replay-from-log-" in path and path.endswith(".json"):
            candidate = resolve_artifact_path(path, ctx.bundle_path.parent)
            break
    if candidate is None:
        return False, "missing replay-from-log report artifact for operational parity"
    if not candidate.exists():
        return False, f"replay-from-log report does not exist: {candidate}"

    payload = json.loads(candidate.read_text(encoding="utf-8"))
    if payload.get("result") != "PASS":
        return False, f"replay-from-log report result is {payload.get('result')}, expected PASS"
    if int(payload.get("drift_count", 1)) != 0:
        return False, f"replay-from-log drift_count must be 0, got {payload.get('drift_count')}"
    if payload.get("profile") != ctx.profile:
        return False, f"replay-from-log profile {payload.get('profile')} != expected {ctx.profile}"
    return True, f"replay-from-log operational parity verified for {ctx.profile}"


def check_attestation(ctx: Context) -> tuple[bool, str]:
    att = ctx.bundle.get("attestationEvidence")
    if not isinstance(att, dict):
        return False, "missing attestationEvidence"

    rank = {"TA0": 0, "TA1": 1, "TA2": 2}
    found = att.get("level")
    if found not in rank:
        return False, f"unknown attestation level {found}"
    if ctx.require_ta not in rank:
        return False, f"unknown required TA level {ctx.require_ta}"

    if rank[found] < rank[ctx.require_ta]:
        return False, f"attestation level {found} below required {ctx.require_ta}"

    if att.get("valid") is not True:
        return False, "attestation valid=false"
    if not str(att.get("deviceIdentity", "")).strip() or not str(att.get("keyId", "")).strip():
        return False, "attestation deviceIdentity/keyId must be present"
    if not is_sha_prefixed(att.get("chainDigest")):
        return False, "attestation chainDigest must be sha256-prefixed"

    policy = ctx.attestation_policy
    allowed_issuers = {str(issuer).replace(" ", "") for issuer in policy.get("allowedIssuers", [])}
    if str(att.get("issuer", "")).replace(" ", "") not in allowed_issuers:
        return False, f"attestation issuer {att.get('issuer')} is not trusted"

    allowed_chain_digests = set(policy.get("allowedChainDigests", []))
    if att.get("chainDigest") not in allowed_chain_digests:
        return False, f"attestation chainDigest {att.get('chainDigest')} is not trusted"

    nonce = str(att.get("nonce", ""))
    nonce_prefix = str(policy.get("requiredNoncePrefix", "nonce-"))
    min_nonce_len = int(policy.get("minimumNonceLength", 8))
    if len(nonce) < min_nonce_len or not nonce.startswith(nonce_prefix):
        return False, f"attestation nonce must start with {nonce_prefix} and be >= {min_nonce_len} chars"

    lifecycle = att.get("keyLifecycle")
    if not isinstance(lifecycle, dict):
        return False, "missing keyLifecycle"

    revocation_prefix = str(policy.get("requiredRevocationEndpointPrefix", "https://"))
    revocation_endpoint = str(lifecycle.get("revocationEndpoint", ""))
    if not revocation_endpoint.startswith(revocation_prefix):
        return False, f"revocation endpoint must start with {revocation_prefix}"
    rotation_days = int(lifecycle.get("rotationDays", 0))
    max_key_age_days = int(lifecycle.get("maxKeyAgeDays", 0))
    if max_key_age_days < rotation_days:
        return False, f"maxKeyAgeDays {max_key_age_days} below rotationDays {rotation_days}"

    trust_roots = resolve_policy_path(ctx.trust_roots, ctx.bundle_path.parent)
    revocation_snapshot_path = resolve_policy_path(ctx.revocation_snapshot, ctx.bundle_path.parent)
    if not trust_roots.exists() or not revocation_snapshot_path.exists():
        return False, "trust roots or revocation snapshot path missing"

    trust_roots_digest = sha256_file_prefixed(trust_roots)
    att_trust_roots_digest = str(att.get("trustRootsDigest", ""))
    if att_trust_roots_digest and att_trust_roots_digest != trust_roots_digest:
        return (
            False,
            f"attestation trustRootsDigest mismatch (expected {trust_roots_digest}, got {att_trust_roots_digest})",
        )

    revocation_digest = sha256_file_prefixed(revocation_snapshot_path)
    att_revocation_digest = str(att.get("revocationSnapshotDigest", ""))
    if att_revocation_digest and att_revocation_digest != revocation_digest:
        return (
            False,
            f"attestation revocationSnapshotDigest mismatch (expected {revocation_digest}, got {att_revocation_digest})",
        )

    attestation_version = str(att.get("attestationVersion", "")).strip()
    if attestation_version and attestation_version != "0.2":
        return False, f"unsupported attestationVersion {attestation_version} (expected 0.2)"
    key_source = str(att.get("keySource", "")).strip()
    if key_source and key_source not in {"FILE", "PKCS11"}:
        return False, f"unsupported attestation keySource {key_source}"

    signer_cert_rel = str(att.get("signerCertificatePath", ""))
    chain_rel = str(att.get("certificateChainPath", ""))
    signer_cert = (
        resolve_artifact_path(signer_cert_rel, ctx.bundle_path.parent)
        if signer_cert_rel
        else resolve_artifact_path(
            "policies/attestation/pki/ta2-signer.cert.pem", ctx.bundle_path.parent
        )
    )
    chain_path = (
        resolve_artifact_path(chain_rel, ctx.bundle_path.parent)
        if chain_rel
        else resolve_artifact_path("policies/attestation/pki/ta2-chain.pem", ctx.bundle_path.parent)
    )
    if not signer_cert.exists() or not chain_path.exists():
        return False, "attestation signer certificate/chain missing"

    ok, message = verify_certificate_chain(signer_cert, chain_path, trust_roots)
    if not ok:
        return False, f"attestation cert chain invalid: {message}"

    evidence_signature = str(att.get("evidenceSignature", ""))
    if evidence_signature:
        if att.get("evidenceSignatureAlgorithm") != "rsa-sha256":
            return (
                False,
                f"unsupported evidenceSignatureAlgorithm {att.get('evidenceSignatureAlgorithm')}",
            )
        ok, message = verify_signature_base64(
            str(att.get("evidenceDigest", "")), evidence_signature, signer_cert
        )
        if not ok:
            return False, f"attestation signature invalid: {message}"

    allowed_signer_certificate_digests = set(policy.get("allowedSignerCertificateDigests", []))
    if allowed_signer_certificate_digests:
        signer_digest = sha256_file_prefixed(signer_cert)
        if signer_digest not in allowed_signer_certificate_digests:
            return False, f"signer certificate digest {signer_digest} is not allowed"

    snapshot_payload = json.loads(revocation_snapshot_path.read_text(encoding="utf-8"))
    version = str(snapshot_payload.get("version", "")).strip()
    if version and version != "0.3.0":
        return False, f"unsupported revocation snapshot version {version}"

    source_digests = snapshot_payload.get("sourceDigests", {})
    if not isinstance(source_digests, dict) or not source_digests:
        return False, "revocation snapshot sourceDigests must not be empty"
    has_crl_source = False
    has_ocsp_source = False
    for source_id, digest in source_digests.items():
        if not is_sha_prefixed(digest):
            return False, f"revocation source digest for {source_id} is not sha256-prefixed"
        lowered = str(source_id).lower()
        if "crl" in lowered:
            has_crl_source = True
        if "ocsp" in lowered:
            has_ocsp_source = True
    if not has_crl_source:
        return False, "revocation snapshot must include at least one CRL source digest"
    if not has_ocsp_source:
        return False, "revocation snapshot must include at least one OCSP source digest"

    ocsp_statuses = snapshot_payload.get("ocspStatuses", [])
    if not isinstance(ocsp_statuses, list) or not ocsp_statuses:
        return False, "revocation snapshot ocspStatuses must not be empty"
    for idx, status in enumerate(ocsp_statuses):
        if not isinstance(status, dict):
            return False, f"ocspStatuses[{idx}] must be an object"
        if not str(status.get("sourceId", "")).strip() or not str(
            status.get("certSerial", "")
        ).strip():
            return False, f"ocspStatuses[{idx}] missing sourceId/certSerial"
        checked = str(status.get("checkedAtUtc", "")).strip()
        checked_ok, checked_or_err = parse_utc(checked)
        if not checked_ok:
            return False, f"ocspStatuses[{idx}] invalid checkedAtUtc: {checked_or_err}"
        state = str(status.get("status", "")).strip()
        if state not in {"good", "revoked", "unknown"}:
            return False, f"ocspStatuses[{idx}] has invalid status {state}"

    snapshot_signer_cert_path = resolve_artifact_path(
        str(snapshot_payload.get("signerCertPath", "")) or str(signer_cert),
        ctx.bundle_path.parent,
    )
    if not snapshot_signer_cert_path.exists():
        return False, f"revocation snapshot signer certificate missing: {snapshot_signer_cert_path}"
    ok, message = verify_certificate_chain(snapshot_signer_cert_path, chain_path, trust_roots)
    if not ok:
        return False, f"revocation snapshot signer cert chain invalid: {message}"

    if snapshot_payload.get("signatureAlgorithm") != "rsa-sha256":
        return (
            False,
            f"unsupported revocation snapshot signatureAlgorithm {snapshot_payload.get('signatureAlgorithm')}",
        )
    signature = str(snapshot_payload.get("signature", "")).strip()
    if not signature:
        return False, "revocation snapshot signature is required"

    snapshot_id_payload = dict(snapshot_payload)
    snapshot_id_payload.pop("snapshotId", None)
    snapshot_id_payload.pop("signature", None)
    snapshot_id_payload.pop("signatureAlgorithm", None)
    snapshot_id_payload.pop("signerCertPath", None)
    expected_snapshot_id = f"sha256:{sha256_hex_str(json.dumps(snapshot_id_payload, sort_keys=True, separators=(',', ':')))}"
    if str(snapshot_payload.get("snapshotId", "")) != expected_snapshot_id:
        return (
            False,
            f"revocation snapshotId mismatch (expected {expected_snapshot_id}, got {snapshot_payload.get('snapshotId')})",
        )
    signable_snapshot = dict(snapshot_payload)
    signable_snapshot.pop("signature", None)
    signable_snapshot.pop("signatureAlgorithm", None)
    signable_snapshot.pop("signerCertPath", None)
    signable_payload = json.dumps(signable_snapshot, sort_keys=True, separators=(",", ":"))
    ok, message = verify_signature_base64(signable_payload, signature, snapshot_signer_cert_path)
    if not ok:
        return False, f"revocation snapshot signature invalid: {message}"

    serial_ok, serial_or_err = certificate_serial_hex(signer_cert)
    if not serial_ok:
        return False, f"failed to read signer cert serial: {serial_or_err}"
    revoked_serials = [
        str(serial).strip().lower() for serial in snapshot_payload.get("revokedSerials", [])
    ]
    if serial_or_err.strip().lower() in revoked_serials:
        return False, f"signer certificate serial {serial_or_err} is revoked"

    generated = str(snapshot_payload.get("generatedAtUtc", "")).strip()
    next_update = str(snapshot_payload.get("nextUpdateUtc", "")).strip()
    if not generated or not next_update:
        return False, "revocation snapshot missing generatedAtUtc/nextUpdateUtc"

    revocation_max_age = int(
        ctx.freshness_policy.get("max_age_hours", {}).get("revocation_snapshot", 720)
    )
    ok, message = check_file_age_hours(
        revocation_snapshot_path, revocation_max_age, "revocation snapshot freshness"
    )
    if not ok:
        return False, message

    if not str(att.get("signingTimeUtc", "")).strip():
        return False, "attestation signingTimeUtc is required"

    for required_usage in policy.get("requiredKeyUsage", []):
        usage_ok, usage_or_err = certificate_text(signer_cert)
    if not usage_ok:
        return False, f"failed to inspect certificate usage: {usage_or_err}"
    for required_usage in policy.get("requiredKeyUsage", []):
        if str(required_usage) not in usage_or_err:
            return False, f"required certificate key usage {required_usage} not present"

    chain_digest = sha256_file_prefixed(chain_path)
    if chain_digest != att.get("chainDigest"):
        return (
            False,
            f"attestation chainDigest mismatch (expected {chain_digest}, got {att.get('chainDigest')})",
        )

    expected_evidence_digest = compute_attestation_evidence_digest(ctx.bundle, att)
    if expected_evidence_digest != att.get("evidenceDigest"):
        return (
            False,
            f"attestation evidenceDigest mismatch (expected {expected_evidence_digest}, got {att.get('evidenceDigest')})",
        )

    generated_ok, generated_or_err = parse_utc(generated)
    if not generated_ok:
        return False, f"invalid generatedAtUtc: {generated_or_err}"
    next_ok, next_or_err = parse_utc(next_update)
    if not next_ok:
        return False, f"invalid nextUpdateUtc: {next_or_err}"
    generated_at = generated_or_err
    next_update_at = next_or_err
    now = datetime.now(timezone.utc)
    if generated_at > next_update_at:
        return False, "revocation snapshot generatedAtUtc is after nextUpdateUtc"
    if now > next_update_at:
        return False, "revocation snapshot nextUpdateUtc has expired"

    signing_time_raw = str(att.get("signingTimeUtc", "")).strip()
    if not signing_time_raw:
        return False, "attestation signingTimeUtc is required"
    signing_ok, signing_or_err = parse_utc(signing_time_raw)
    if not signing_ok:
        return False, f"invalid attestation signingTimeUtc: {signing_or_err}"
    signing_time = signing_or_err
    if signing_time > now:
        return False, "attestation signingTimeUtc cannot be in the future"
    if signing_time < generated_at:
        return False, "attestation signingTimeUtc predates revocation snapshot generation"

    if found == "TA2":
        if att.get("certificateProfile") not in {"HSM_CERTIFIED", "SECURE_ELEMENT_CERTIFIED"}:
            return False, f"invalid TA2 certificate profile {att.get('certificateProfile')}"

        claims = att.get("claims")
        if not isinstance(claims, dict):
            return False, "missing attestation claims"
        if claims.get("hsmBacked") is not True:
            return False, "TA2 requires hsmBacked=true"
        if claims.get("secureBootMeasured") is not True:
            return False, "TA2 requires secureBootMeasured=true"
        if rotation_days > 365:
            return False, f"TA2 rotationDays {rotation_days} exceeds max 365"
        if max_key_age_days > 365:
            return False, f"TA2 maxKeyAgeDays {max_key_age_days} exceeds max 365"

    if found == "TA2" and key_source != "PKCS11":
        return False, f"TA2 attestation requires keySource=PKCS11, got {key_source}"

    return True, f"attestation {found} satisfies minimum {ctx.require_ta}"


def check_incident_windows(bundle: dict[str, Any]) -> tuple[bool, str]:
    initial = bundle.get("incidentInitialTemplate")
    pack = bundle.get("incidentPackTemplate")
    if not isinstance(initial, dict):
        return False, "missing incidentInitialTemplate"
    if not isinstance(pack, dict):
        return False, "missing incidentPackTemplate"

    windows = initial.get("reportingWindowsDays")
    if not isinstance(windows, dict):
        return False, "incidentInitialTemplate missing reportingWindowsDays"

    standard = windows.get("standard")
    widespread = windows.get("widespread")
    fatal = windows.get("fatal")
    if (standard, widespread, fatal) != (15, 2, 10):
        return False, f"incident windows must be 15/2/10, got {standard}/{widespread}/{fatal}"

    if int(pack.get("fullPackSlaDays", 999)) > 10:
        return False, f"fullPackSlaDays {pack.get('fullPackSlaDays')} exceeds max 10"

    return True, "incident window policy checks passed"


def verify_transparency_proof(
    proof: dict[str, Any],
    trusted_map: dict[str, dict[str, Any]],
    expected_log: str,
    policy: dict[str, Any],
    declared_bundle_digest: str | None,
    freshness_policy: dict[str, Any],
    repo_root: Path,
) -> tuple[bool, str]:
    proof_version = str(proof.get("proofVersion", "")).strip()
    if proof_version and proof_version not in {"0.1", "0.2"}:
        return False, f"unsupported proofVersion {proof_version} for {expected_log}"

    if proof.get("logId") != expected_log:
        return False, f"proof log id {proof.get('logId')} != expected {expected_log}"

    if proof.get("verifiedOffline") is not True:
        return False, "proof must set verifiedOffline=true"
    if policy.get("require_live_publication") and str(proof.get("source", "")) != "live":
        return (
            False,
            f"proof source must be live when policy requires live publication, got {proof.get('source')}",
        )
    if not str(proof.get("entryUuid", "")).strip():
        return False, "entryUuid is required"
    if not str(proof.get("logUrl", "")).strip():
        return False, "logUrl is required"
    root_hash = str(proof.get("rootHash", "")).strip()
    leaf_hash_value = str(proof.get("leafHash", "")).strip()
    if not is_sha_prefixed(root_hash):
        return False, "rootHash must be sha256-prefixed"
    if not is_sha_prefixed(leaf_hash_value):
        return False, "leafHash must be sha256-prefixed"
    tree_size = int(proof.get("treeSize", 0))
    log_index = int(proof.get("logIndex", 0))
    if tree_size <= 0:
        return False, "treeSize must be > 0"
    if log_index >= tree_size:
        return False, f"logIndex {log_index} must be < treeSize {tree_size}"
    checkpoint_raw = str(proof.get("checkpoint", ""))
    checkpoint = checkpoint_raw.strip()
    if policy.get("require_checkpoint_text") and not checkpoint:
        return False, "checkpoint text is required"
    expected_checkpoint_hash = f"sha256:{sha256_hex_str(checkpoint_raw)}"
    if str(proof.get("checkpointHash", "")) != expected_checkpoint_hash:
        return (
            False,
            f"checkpointHash mismatch (expected {expected_checkpoint_hash}, got {proof.get('checkpointHash')})",
        )
    source = str(proof.get("source", "")).strip()
    if source and source not in {"live", "local"}:
        return False, f"unsupported proof source {source}"

    inclusion_path = proof.get("inclusionPath", [])
    consistency_path = proof.get("consistencyPath", [])
    if not isinstance(inclusion_path, list) or not isinstance(consistency_path, list):
        return False, "inclusionPath/consistencyPath must be arrays"
    for idx, digest in enumerate(inclusion_path):
        if not is_sha_prefixed(digest):
            return False, f"inclusionPath[{idx}] is not sha256-prefixed"
    for idx, digest in enumerate(consistency_path):
        if not is_sha_prefixed(digest):
            return False, f"consistencyPath[{idx}] is not sha256-prefixed"

    integrated_time = str(proof.get("integratedTimeUtc", ""))
    if integrated_time and not integrated_time.endswith("Z"):
        return False, f"integratedTimeUtc must be UTC (Z suffix), got {integrated_time}"
    if int(policy.get("max_checkpoint_age_hours", 0)) > 0 and not integrated_time.strip():
        return False, "integratedTimeUtc required by transparency freshness policy"

    trusted = trusted_map.get(expected_log)
    if trusted is None:
        return False, f"trusted checkpoint missing for {expected_log}"

    trusted_endpoint = str(trusted.get("endpoint", "")).strip()
    if trusted_endpoint and not str(proof.get("logUrl", "")).startswith(trusted_endpoint):
        return (
            False,
            f"proof logUrl {proof.get('logUrl')} does not match trusted endpoint {trusted_endpoint}",
        )

    trusted_checkpoint_hash = str(trusted.get("checkpointHash", "")).strip()
    if trusted_checkpoint_hash and proof.get("checkpointHash") != trusted_checkpoint_hash:
        return (
            False,
            f"checkpoint hash mismatch for {expected_log} (expected {trusted_checkpoint_hash}, got {proof.get('checkpointHash')})",
        )

    for label in ["entryDigest", "inclusionProofHash", "consistencyProofHash"]:
        if not is_sha_prefixed(proof.get(label)):
            return False, f"{label} for {expected_log} is not sha256-prefixed"

    inclusion_expected = f"sha256:{sha256_hex_bytes(json.dumps({'hashes': inclusion_path, 'rootHash': root_hash}, sort_keys=True, separators=(',', ':')).encode('utf-8'))}"
    if proof.get("inclusionProofHash") != inclusion_expected:
        return (
            False,
            f"inclusionProofHash mismatch (expected {inclusion_expected}, got {proof.get('inclusionProofHash')})",
        )
    consistency_expected = f"sha256:{sha256_hex_bytes(json.dumps({'hashes': consistency_path, 'rootHash': root_hash}, sort_keys=True, separators=(',', ':')).encode('utf-8'))}"
    if proof.get("consistencyProofHash") != consistency_expected:
        return (
            False,
            f"consistencyProofHash mismatch (expected {consistency_expected}, got {proof.get('consistencyProofHash')})",
        )

    signature_algorithm = str(trusted.get("signatureAlgorithm", "")).strip() or "rsa-sha256"
    signature_encoding = str(trusted.get("signatureEncoding", "")).strip() or "base64"
    if signature_algorithm != "rsa-sha256" or signature_encoding != "base64":
        return (
            False,
            f"unsupported trusted log signature settings for {expected_log}: {signature_algorithm}/{signature_encoding}",
        )

    signer_cert = resolve_artifact_path(
        str(trusted.get("signerCertificatePath", "")),
        repo_root,
    )
    chain_path = resolve_artifact_path(
        str(trusted.get("certificateChainPath", "")),
        repo_root,
    )
    trust_roots = resolve_artifact_path(
        str(trusted.get("trustRootsPath", "")),
        repo_root,
    )
    if not signer_cert.exists() or not chain_path.exists() or not trust_roots.exists():
        return False, f"trusted log material missing for {expected_log}"

    ok, message = verify_certificate_chain(signer_cert, chain_path, trust_roots)
    if not ok:
        return False, f"transparency signer chain invalid for {expected_log}: {message}"
    payload = transparency_signature_payload(proof)
    ok, message = verify_signature_base64(payload, str(proof.get("signature", "")), signer_cert)
    if not ok:
        return False, f"transparency signature invalid for {expected_log}: {message}"

    if policy.get("require_entry_digest_matches_bundle"):
        if declared_bundle_digest is None:
            return False, "declared bundle digest missing for proof binding"
        if proof.get("entryDigest") != declared_bundle_digest:
            return (
                False,
                f"entryDigest {proof.get('entryDigest')} does not match declared bundle digest {declared_bundle_digest}",
            )

    if int(policy.get("max_checkpoint_age_hours", 0)) > 0:
        integrated_ok, integrated_or_err = parse_utc(integrated_time)
        if not integrated_ok:
            return False, f"invalid integratedTimeUtc: {integrated_or_err}"
        integrated_at = integrated_or_err
        now = datetime.now(timezone.utc)
        if integrated_at > now:
            return False, f"integratedTimeUtc {integrated_time} cannot be in the future"
        age_hours = int((now - integrated_at).total_seconds() // 3600)
        max_hours = int(
            freshness_policy.get("max_age_hours", {}).get(
                "transparency_proof",
                int(policy.get("max_checkpoint_age_hours", 0)),
            )
        )
        if age_hours > max_hours:
            return (
                False,
                f"transparency proof for {expected_log} exceeded max age ({age_hours}h > {max_hours}h)",
            )

    return True, f"{expected_log} transparency proof verified"


def check_named_proof(ctx: Context, expected_log: str) -> tuple[bool, str]:
    required_logs = ctx.transparency_policy.get("required_logs", [])
    if required_logs and expected_log not in required_logs:
        return True, f"{expected_log} not required by transparency policy"
    if expected_log not in ctx.require_transparency:
        return True, f"{expected_log} not required by invocation"

    proofs = ctx.bundle.get("transparencyProofs")
    if not isinstance(proofs, dict):
        return False, "missing transparencyProofs"

    proof = proofs.get(expected_log)
    if not isinstance(proof, dict):
        return False, f"missing transparency proof for {expected_log}"

    declared_bundle_digest = None
    lineage = ctx.bundle.get("lineage")
    if isinstance(lineage, dict):
        value = lineage.get("assurancePackDigest")
        if isinstance(value, str):
            declared_bundle_digest = value
    if declared_bundle_digest is None:
        conformance_report = ctx.bundle.get("conformanceReport")
        if isinstance(conformance_report, dict):
            value = conformance_report.get("bundleDigest")
            if isinstance(value, str):
                declared_bundle_digest = value

    if expected_log == "mirror" and ctx.transparency_policy.get("require_mirror_parity"):
        rekor_digest = str(pointer(ctx.bundle, "/transparencyProofs/rekor/entryDigest") or "")
        mirror_digest = str(proof.get("entryDigest", ""))
        if rekor_digest and rekor_digest != mirror_digest:
            return (
                False,
                f"mirror parity mismatch: mirror entryDigest {mirror_digest} != rekor entryDigest {rekor_digest}",
            )

    return verify_transparency_proof(
        proof,
        ctx.trusted_map,
        expected_log,
        ctx.transparency_policy,
        declared_bundle_digest,
        ctx.freshness_policy,
        ctx.repo_root,
    )


def check_badge_status(ctx: Context) -> tuple[bool, str]:
    badge = ctx.bundle.get("badgeEntry")
    if not isinstance(badge, dict):
        return False, "missing badgeEntry"

    badge_id = badge.get("badgeId")
    status = badge.get("status")
    if not badge_id:
        return False, "bundle badgeId missing"
    if status != "active":
        return False, f"bundle badge status {status} is not active"

    entries = ctx.badge_registry.get("entries", [])
    entry = next((item for item in entries if item.get("badgeId") == badge_id), None)
    if entry is None:
        return False, f"badge {badge_id} not found in registry"

    if entry.get("status") != "active":
        return False, f"registry badge {badge_id} status is {entry.get('status')}"

    return True, f"badge {badge_id} is active"


def evaluate_check(check_id: str, ctx: Context) -> tuple[bool, str]:
    bundle = ctx.bundle

    if check_id == "CHK_SCHEMA_ASSURANCEPACK":
        return check_required_paths(
            bundle,
            [
                "/assurancePackVersion",
                "/profile",
                "/policy",
                "/specVersion",
                "/evidenceMap",
                "/conformanceReport",
                "/signedOperationalLog",
                "/replayRecipe",
                "/attestationEvidence",
                "/incidentInitialTemplate",
                "/incidentPackTemplate",
                "/transparencyProofs/rekor",
                "/transparencyProofs/mirror",
                "/badgeEntry",
                "/artifacts",
            ],
            "assurance pack envelope",
        )
    if check_id == "CHK_SCHEMA_EVIDENCEMAP":
        return check_required_paths(
            bundle,
            [
                "/evidenceMap/evidenceMapVersion",
                "/evidenceMap/system",
                "/evidenceMap/tasc",
                "/evidenceMap/topology",
                "/evidenceMap/logs",
                "/evidenceMap/replay",
                "/evidenceMap/attestation",
                "/evidenceMap/incident",
            ],
            "evidence map",
        )
    if check_id == "CHK_SCHEMA_CONFORMANCE_REPORT":
        return check_required_paths(
            bundle,
            [
                "/conformanceReport/tascVerifyVersion",
                "/conformanceReport/specVersion",
                "/conformanceReport/result",
                "/conformanceReport/checks",
            ],
            "conformance report",
        )
    if check_id == "CHK_SCHEMA_SIGNED_LOG":
        return check_required_paths(
            bundle,
            [
                "/signedOperationalLog/schemaVersion",
                "/signedOperationalLog/signerKeyId",
                "/signedOperationalLog/signature",
                "/signedOperationalLog/root",
                "/signedOperationalLog/events",
            ],
            "signed operational log",
        )
    if check_id == "CHK_SCHEMA_REPLAY_RECIPE":
        return check_required_paths(
            bundle,
            [
                "/replayRecipe/recipeVersion",
                "/replayRecipe/seedsDigest",
                "/replayRecipe/buildDigest",
                "/replayRecipe/configDigest",
                "/replayRecipe/environmentDigest",
                "/replayRecipe/containerDigest",
            ],
            "replay recipe",
        )
    if check_id == "CHK_SCHEMA_ATTESTATION":
        return check_required_paths(
            bundle,
            [
                "/attestationEvidence/level",
                "/attestationEvidence/certificateProfile",
                "/attestationEvidence/keyLifecycle/rotationDays",
                "/attestationEvidence/claims/hsmBacked",
            ],
            "attestation evidence",
        )
    if check_id == "CHK_SCHEMA_INCIDENT_INITIAL":
        return check_required_paths(
            bundle,
            [
                "/incidentInitialTemplate/templateId",
                "/incidentInitialTemplate/reportingWindowsDays/standard",
                "/incidentInitialTemplate/reportingWindowsDays/widespread",
                "/incidentInitialTemplate/reportingWindowsDays/fatal",
            ],
            "incident initial template",
        )
    if check_id == "CHK_SCHEMA_INCIDENT_PACK":
        return check_required_paths(
            bundle,
            [
                "/incidentPackTemplate/templateId",
                "/incidentPackTemplate/requiredArtifacts",
                "/incidentPackTemplate/fullPackSlaDays",
            ],
            "incident full pack template",
        )
    if check_id == "CHK_SCHEMA_TRANSPARENCY_REKOR":
        return check_required_paths(
            bundle,
            [
                "/transparencyProofs/rekor/logId",
                "/transparencyProofs/rekor/logUrl",
                "/transparencyProofs/rekor/entryUuid",
                "/transparencyProofs/rekor/logIndex",
                "/transparencyProofs/rekor/treeSize",
                "/transparencyProofs/rekor/rootHash",
                "/transparencyProofs/rekor/leafHash",
                "/transparencyProofs/rekor/checkpoint",
                "/transparencyProofs/rekor/checkpointHash",
                "/transparencyProofs/rekor/inclusionPath",
                "/transparencyProofs/rekor/consistencyPath",
                "/transparencyProofs/rekor/inclusionProofHash",
                "/transparencyProofs/rekor/consistencyProofHash",
            ],
            "rekor transparency proof",
        )
    if check_id == "CHK_SCHEMA_TRANSPARENCY_MIRROR":
        return check_required_paths(
            bundle,
            [
                "/transparencyProofs/mirror/logId",
                "/transparencyProofs/mirror/logUrl",
                "/transparencyProofs/mirror/entryUuid",
                "/transparencyProofs/mirror/logIndex",
                "/transparencyProofs/mirror/treeSize",
                "/transparencyProofs/mirror/rootHash",
                "/transparencyProofs/mirror/leafHash",
                "/transparencyProofs/mirror/checkpoint",
                "/transparencyProofs/mirror/checkpointHash",
                "/transparencyProofs/mirror/inclusionPath",
                "/transparencyProofs/mirror/consistencyPath",
                "/transparencyProofs/mirror/inclusionProofHash",
                "/transparencyProofs/mirror/consistencyProofHash",
            ],
            "mirror transparency proof",
        )
    if check_id == "CHK_SCHEMA_BADGE_ENTRY":
        return check_required_paths(
            bundle,
            [
                "/badgeEntry/badgeId",
                "/badgeEntry/status",
                "/badgeEntry/badgeType",
            ],
            "badge entry",
        )
    if check_id == "CHK_PROFILE_MATCH":
        return check_bundle_value(bundle, "/profile", ctx.profile, "profile")
    if check_id == "CHK_POLICY_MATCH":
        return check_bundle_value(bundle, "/policy", ctx.policy, "policy")
    if check_id == "CHK_TOPOLOGY_INTERLOCK_MEDIATION":
        return check_interlock_mediation(bundle)
    if check_id == "CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY":
        return check_authority_exclusivity(bundle)
    if check_id == "CHK_TOPOLOGY_ASSERTION_REFS":
        return check_assertion_refs(bundle)
    if check_id == "CHK_ARTIFACT_HASH_INTEGRITY":
        return check_artifact_hashes(ctx)
    if check_id == "CHK_LOG_HASH_CHAIN":
        return check_log_hash_chain(bundle)
    if check_id == "CHK_LOG_SIGNATURE":
        return check_log_signature(ctx)
    if check_id == "CHK_RETENTION_EU_MINIMUM":
        return check_retention_minimum(bundle, 6)
    if check_id == "CHK_REPLAY_COMPLETENESS":
        return check_replay(ctx)
    if check_id == "CHK_REPLAY_OPERATIONAL_PARITY":
        return check_replay_operational_parity(ctx)
    if check_id == "CHK_ATTESTATION_TA2":
        return check_attestation(ctx)
    if check_id == "CHK_INCIDENT_WINDOWS_EU":
        return check_incident_windows(bundle)
    if check_id == "CHK_TRANSPARENCY_REKOR_PROOF":
        return check_named_proof(ctx, "rekor")
    if check_id == "CHK_TRANSPARENCY_MIRROR_PROOF":
        return check_named_proof(ctx, "mirror")
    if check_id == "CHK_BADGE_ACTIVE_NOT_REVOKED":
        return check_badge_status(ctx)

    return False, f"unknown check id {check_id}"


def load_checks(path: Path) -> list[str]:
    catalog = yaml.safe_load(path.read_text(encoding="utf-8"))
    checks = catalog.get("checks", [])
    return [item["id"] for item in checks if isinstance(item, dict) and "id" in item]


def run_profile_checks(
    repo_root: Path,
    bundle_path: Path,
    profile: str,
    policy: str,
    require_ta: str,
    require_transparency: list[str],
    check_ids: list[str],
    trusted_map: dict[str, dict[str, Any]],
    badge_registry: dict[str, Any],
    attestation_policy: dict[str, Any],
    trust_roots: Path,
    revocation_snapshot: Path,
    freshness_policy: dict[str, Any],
    transparency_policy: dict[str, Any],
) -> ProfileReport:
    if not bundle_path.exists():
        return ProfileReport(
            profile=profile,
            bundle=str(bundle_path),
            result="FAIL",
            checks=[],
            failedChecks=["CHK_BUNDLE_FILE_EXISTS"],
        )

    bundle = json.loads(bundle_path.read_text(encoding="utf-8"))
    ctx = Context(
        repo_root=repo_root,
        bundle_path=bundle_path,
        bundle=bundle,
        profile=profile,
        policy=policy,
        require_ta=require_ta,
        require_transparency=require_transparency,
        trusted_map=trusted_map,
        badge_registry=badge_registry,
        attestation_policy=attestation_policy,
        trust_roots=trust_roots,
        revocation_snapshot=revocation_snapshot,
        freshness_policy=freshness_policy,
        transparency_policy=transparency_policy,
    )

    checks: list[CheckResult] = []
    failed: list[str] = []

    for check_id in check_ids:
        try:
            ok, message = evaluate_check(check_id, ctx)
        except Exception as exc:  # defensive, keeps smoke report deterministic
            ok, message = False, f"exception during check: {exc}"
        checks.append(CheckResult(id=check_id, result="PASS" if ok else "FAIL", message=message))
        if not ok:
            failed.append(check_id)

    return ProfileReport(
        profile=profile,
        bundle=str(bundle_path),
        result="PASS" if not failed else "FAIL",
        checks=checks,
        failedChecks=failed,
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--fixtures-dir", default="conformance/fixtures/tasc")
    parser.add_argument("--checks-file", default="spec/tasc/checks.yaml")
    parser.add_argument(
        "--trusted-checkpoints",
        default="policies/transparency/trusted-log-checkpoints.json",
    )
    parser.add_argument("--badge-registry", default="policies/badge-registry.json")
    parser.add_argument(
        "--attestation-trust-policy",
        default="policies/attestation/trust-policy.json",
    )
    parser.add_argument("--trust-roots", default="policies/attestation/pki/trust-roots.pem")
    parser.add_argument(
        "--revocation-snapshot",
        default="policies/attestation/revocation-snapshot.json",
    )
    parser.add_argument(
        "--freshness-policy",
        default="policies/provenance/freshness-policy.yaml",
    )
    parser.add_argument(
        "--transparency-policy",
        default="policies/transparency/verification-policy.yaml",
    )
    parser.add_argument("--policy", default="eu-north-star")
    parser.add_argument("--require-ta", default="TA2")
    parser.add_argument("--require-transparency", default="rekor,mirror")
    parser.add_argument("--profiles", default="uas-small,fixed-wing,hybrid-vtol")
    parser.add_argument(
        "--output",
        default="conformance/reports/tasc-offline-smoke.json",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    fixtures_dir = (repo_root / args.fixtures_dir).resolve()

    check_ids = load_checks((repo_root / args.checks_file).resolve())
    trusted_logs = json.loads((repo_root / args.trusted_checkpoints).read_text(encoding="utf-8"))
    trusted_map = {entry["id"]: entry for entry in trusted_logs.get("logs", [])}
    badge_registry = json.loads((repo_root / args.badge_registry).read_text(encoding="utf-8"))
    attestation_policy = json.loads(
        (repo_root / args.attestation_trust_policy).read_text(encoding="utf-8")
    )
    freshness_policy = yaml.safe_load(
        (repo_root / args.freshness_policy).read_text(encoding="utf-8")
    ) or {}
    transparency_policy = yaml.safe_load(
        (repo_root / args.transparency_policy).read_text(encoding="utf-8")
    ) or {}
    trust_roots = (repo_root / args.trust_roots).resolve()
    revocation_snapshot = (repo_root / args.revocation_snapshot).resolve()

    profiles = [part.strip() for part in args.profiles.split(",") if part.strip()]
    required_transparency = parse_csv_list(args.require_transparency)

    profile_reports: list[ProfileReport] = []
    for profile in profiles:
        bundle_path = fixtures_dir / f"tasc-assurance-pack-{profile}.json"
        report = run_profile_checks(
            repo_root=repo_root,
            bundle_path=bundle_path,
            profile=profile,
            policy=args.policy,
            require_ta=args.require_ta,
            require_transparency=required_transparency,
            check_ids=check_ids,
            trusted_map=trusted_map,
            badge_registry=badge_registry,
            attestation_policy=attestation_policy,
            trust_roots=trust_roots,
            revocation_snapshot=revocation_snapshot,
            freshness_policy=freshness_policy,
            transparency_policy=transparency_policy,
        )
        profile_reports.append(report)

    overall_failed = any(report.result == "FAIL" for report in profile_reports)
    output_payload = {
        "offlineSmokeVersion": "0.1.0",
        "policy": args.policy,
        "requireTa": args.require_ta,
        "requireTransparency": required_transparency,
        "checks": check_ids,
        "profiles": [
            {
                "profile": report.profile,
                "bundle": report.bundle,
                "result": report.result,
                "checks": [
                    {"id": check.id, "result": check.result, "message": check.message}
                    for check in report.checks
                ],
                "failedChecks": report.failedChecks,
            }
            for report in profile_reports
        ],
        "result": "FAIL" if overall_failed else "PASS",
    }

    output_path = (repo_root / args.output).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(output_payload, indent=2) + "\n", encoding="utf-8")

    for report in profile_reports:
        print(f"{report.profile}: {report.result} ({len(report.failedChecks)} failed checks)")
    print(f"overall: {output_payload['result']}")
    print(f"report: {output_path}")

    return 1 if overall_failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
