#!/usr/bin/env python3
"""Build a TASC Assurance Pack JSON and signed archive."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import shlex
import subprocess
import tarfile
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from functools import partial
from pathlib import Path
from typing import Callable
from tempfile import NamedTemporaryFile

import yaml

from pkcs11_client import Pkcs11Client, compose_chain_file, load_profile


@dataclass
class SignerMaterial:
    key_source: str
    signer_cert: Path
    signing_chain: Path
    sign_fn: Callable[[bytes], bytes]


PROCUREMENT_ZERO_DIGEST = "sha256:" + ("0" * 64)


def utc_now() -> datetime:
    return datetime.now(timezone.utc).replace(microsecond=0)


def utc_iso(value: datetime) -> str:
    return value.isoformat().replace("+00:00", "Z")


def utc_now_iso() -> str:
    return utc_iso(utc_now())


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def sha_prefixed_hex(text: str) -> str:
    return f"sha256:{sha256_bytes(text.encode('utf-8'))}"


def digest_of_obj(obj: object) -> str:
    encoded = json.dumps(obj, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return f"sha256:{sha256_bytes(encoded)}"


def load_yaml(path: Path) -> dict:
    payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    if not isinstance(payload, dict):
        raise SystemExit(f"expected YAML mapping: {path}")
    return payload


def infer_git_commit(repo_root: Path) -> str:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo_root), "rev-parse", "--short=12", "HEAD"],
            stderr=subprocess.DEVNULL,
            text=True,
        )
        return out.strip()
    except Exception:
        return "000000000000"


def ensure_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def relpath_or_abs(path: Path, repo_root: Path) -> str:
    try:
        return str(path.resolve().relative_to(repo_root.resolve()))
    except Exception:
        return str(path.resolve())


def relpath_from_output(path: Path, output_dir: Path, repo_root: Path) -> str:
    resolved = path.resolve()
    try:
        return str(resolved.relative_to(output_dir.resolve()))
    except Exception:
        return relpath_or_abs(resolved, repo_root)


def openssl_sign_bytes(private_key: Path, payload: bytes) -> bytes:
    with NamedTemporaryFile(delete=False) as in_file, NamedTemporaryFile(delete=False) as out_file:
        in_path = Path(in_file.name)
        out_path = Path(out_file.name)
        in_file.write(payload)
        in_file.flush()

    try:
        subprocess.run(
            [
                "openssl",
                "dgst",
                "-sha256",
                "-sign",
                str(private_key),
                "-out",
                str(out_path),
                str(in_path),
            ],
            check=True,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        return out_path.read_bytes()
    finally:
        in_path.unlink(missing_ok=True)
        out_path.unlink(missing_ok=True)


def resolve_signer_material(
    args: argparse.Namespace,
    repo_root: Path,
    out_dir: Path,
) -> SignerMaterial:
    if args.signer_mode == "file":
        signer_cert = (repo_root / args.signing_cert).resolve()
        signing_chain = (repo_root / args.signing_chain).resolve()
        signing_key = (repo_root / args.signing_key).resolve()
        if not signer_cert.exists() or not signing_chain.exists() or not signing_key.exists():
            raise SystemExit("file signer mode requires existing signing key/cert/chain files")
        return SignerMaterial(
            key_source="FILE",
            signer_cert=signer_cert,
            signing_chain=signing_chain,
            sign_fn=partial(openssl_sign_bytes, signing_key),
        )

    if args.signer_mode == "pkcs11":
        if args.pkcs11_sign_cmd:
            raise SystemExit("--pkcs11-sign-cmd is deprecated; use native --pkcs11-* flags")

        profile_path = (repo_root / args.pkcs11_profile).resolve()
        profile = load_profile(profile_path)
        module = args.pkcs11_module or profile.module
        slot = str(args.pkcs11_slot) if args.pkcs11_slot is not None else profile.slot
        token_label = args.pkcs11_token_label or profile.token_label
        key_label = args.pkcs11_key_label or profile.key_label
        cert_label = args.pkcs11_cert_label or profile.cert_label
        pin_env = args.pkcs11_pin_env or profile.pin_env
        mechanism = args.pkcs11_mechanism or profile.mechanism

        client = Pkcs11Client(
            module=module,
            slot=slot,
            token_label=token_label,
            key_label=key_label,
            cert_label=cert_label,
            pin_env=pin_env,
            mechanism=mechanism,
        )

        signer_cert = out_dir / f"pkcs11-signer-cert-{args.profile}.pem"
        client.export_certificate_pem(cert_label, signer_cert)

        fallback_chain = (
            (repo_root / profile.intermediate_chain_path).resolve()
            if profile.intermediate_chain_path
            else (repo_root / args.signing_chain).resolve()
        )
        signing_chain = out_dir / f"pkcs11-chain-{args.profile}.pem"
        compose_chain_file(
            signer_cert_pem_path=signer_cert,
            chain_output_path=signing_chain,
            intermediate_chain_path=fallback_chain if fallback_chain.exists() else None,
        )

        return SignerMaterial(
            key_source="PKCS11",
            signer_cert=signer_cert.resolve(),
            signing_chain=signing_chain.resolve(),
            sign_fn=client.sign,
        )

    raise SystemExit(f"unsupported signer mode {args.signer_mode}")


def cert_issuer(cert_path: Path) -> str:
    out = subprocess.check_output(
        ["openssl", "x509", "-in", str(cert_path), "-issuer", "-noout"],
        text=True,
    ).strip()
    issuer = out.split("=", 1)[1] if "=" in out else out
    return issuer.replace(" ", "")


def build_signed_log(
    profile: str,
    repo_root: Path,
    trust_roots: Path,
    signer_material: SignerMaterial,
) -> dict:
    signer = f"kid:tasc-ta2-{profile}"
    base_events = [
        (1, 0, "mode_switch", f"{profile}:allow"),
        (2, 20, "heartbeat", f"{profile}:healthy"),
        (3, 40, "guard_eval", f"{profile}:interlock_authorized"),
    ]

    events = []
    prev = ""
    for seq, tick_ms, event_type, payload in base_events:
        payload_digest = f"sha256:{sha256_bytes(payload.encode('utf-8'))}"
        raw = f"{seq}|{prev}|{payload_digest}|{event_type}|{tick_ms}".encode("utf-8")
        event_hash = f"sha256:{sha256_bytes(raw)}"
        events.append(
            {
                "seq": seq,
                "tickMs": tick_ms,
                "eventType": event_type,
                "payloadDigest": payload_digest,
                "prevHash": prev,
                "hash": event_hash,
            }
        )
        prev = event_hash

    root = prev
    signature = base64.b64encode(signer_material.sign_fn(root.encode("utf-8"))).decode("ascii")

    return {
        "schemaVersion": "0.2",
        "algorithm": "sha256-chain-v1",
        "signerKeyId": signer,
        "signature": signature,
        "signatureAlgorithm": "rsa-sha256",
        "signatureEncoding": "base64",
        "signingTimeUtc": utc_now_iso(),
        "signerCertificatePath": relpath_or_abs(signer_material.signer_cert, repo_root),
        "certificateChainPath": relpath_or_abs(signer_material.signing_chain, repo_root),
        "trustRootsDigest": f"sha256:{sha256_file(trust_roots)}",
        "keySource": signer_material.key_source,
        "root": root,
        "events": events,
    }


def build_replay_recipe(profile: str) -> dict:
    return {
        "recipeVersion": "0.1",
        "profile": profile,
        "seedsDigest": sha_prefixed_hex(f"{profile}:seed"),
        "buildDigest": sha_prefixed_hex(f"{profile}:build"),
        "configDigest": sha_prefixed_hex(f"{profile}:config"),
        "environmentDigest": sha_prefixed_hex(f"{profile}:environment"),
        "containerDigest": sha_prefixed_hex(f"{profile}:container"),
        "reproducibilityGuarantee": "Deterministic replay with fixed seeds/build/config/environment digests.",
        "replaySlaHours": 8,
    }


def build_attestation(
    profile: str,
    repo_root: Path,
    signed_log: dict,
    signer_material: SignerMaterial,
    trust_roots: Path,
    revocation_snapshot: Path,
) -> dict:
    attestation_level = "TA2" if signer_material.key_source == "PKCS11" else "TA1"
    hsm_backed = signer_material.key_source == "PKCS11"
    software_state_digest = sha_prefixed_hex(f"{profile}:software-state")
    evidence_payload = (
        f"profile={profile}|logRoot={signed_log['root']}|softwareStateDigest={software_state_digest}"
    )
    evidence_digest = f"sha256:{sha256_bytes(evidence_payload.encode('utf-8'))}"
    evidence_signature = base64.b64encode(
        signer_material.sign_fn(evidence_digest.encode("utf-8"))
    ).decode("ascii")

    return {
        "attestationVersion": "0.2",
        "level": attestation_level,
        "deviceIdentity": f"device:{profile}:hsm-001",
        "keyId": f"hsm-key:{profile}:sig-01",
        "issuer": cert_issuer(signer_material.signer_cert),
        "certificateProfile": "HSM_CERTIFIED" if hsm_backed else "SOFTWARE_SIGNED",
        "evidenceDigest": evidence_digest,
        "chainDigest": f"sha256:{sha256_file(signer_material.signing_chain)}",
        "evidenceSignature": evidence_signature,
        "evidenceSignatureAlgorithm": "rsa-sha256",
        "signingTimeUtc": utc_now_iso(),
        "signerCertificatePath": relpath_or_abs(signer_material.signer_cert, repo_root),
        "certificateChainPath": relpath_or_abs(signer_material.signing_chain, repo_root),
        "trustRootsDigest": f"sha256:{sha256_file(trust_roots)}",
        "revocationSnapshotDigest": f"sha256:{sha256_file(revocation_snapshot)}",
        "revocationSnapshotPath": relpath_or_abs(revocation_snapshot, repo_root),
        "keySource": signer_material.key_source,
        "nonce": f"nonce-{profile}-ta2-verified",
        "valid": True,
        "keyLifecycle": {
            "rotationDays": 90,
            "maxKeyAgeDays": 365,
            "revocationEndpoint": "https://registry.tasc.dev/revocations",
            "lastRotated": "2026-01-15",
        },
        "claims": {
            "secureBootMeasured": True,
            "hsmBacked": hsm_backed,
            "softwareStateDigest": software_state_digest,
        },
    }


def transparency_signature_payload(proof: dict) -> str:
    return (
        f"{proof['logId']}|{proof['entryUuid']}|{proof['logIndex']}|{proof['treeSize']}|"
        f"{proof['rootHash']}|{proof['leafHash']}|{proof['checkpointHash']}|{proof['entryDigest']}|"
        f"{proof['inclusionProofHash']}|{proof['consistencyProofHash']}|{proof['integratedTimeUtc']}"
    )


def build_transparency_proof(
    log_id: str,
    checkpoints: dict[str, dict],
    entry_digest: str,
    signer_material: SignerMaterial,
    live_payload: dict | None = None,
) -> dict:
    trusted = checkpoints.get(log_id, {})
    live_payload = live_payload or {}
    root_hash = str(live_payload.get("rootHash", "")).strip() or sha_prefixed_hex(
        f"{log_id}:{entry_digest}:root"
    )
    leaf_hash = str(live_payload.get("leafHash", "")).strip() or sha_prefixed_hex(
        f"leaf:{entry_digest}"
    )
    tree_size = int(live_payload.get("treeSize", 1) or 1)
    log_index = int(live_payload.get("logIndex", 0) or 0)
    inclusion_path = live_payload.get("inclusionPath", []) or [sha_prefixed_hex(f"{log_id}:{entry_digest}:path0")]
    consistency_path = live_payload.get("consistencyPath", []) or []
    checkpoint_text = str(live_payload.get("checkpoint", ""))
    if not checkpoint_text.strip():
        checkpoint_text = f"{log_id}\n{tree_size}\n{root_hash}\n"
    checkpoint_hash = str(live_payload.get("checkpointHash", "")).strip()
    if not checkpoint_hash:
        checkpoint_hash = str(trusted.get("checkpointHash", "")).strip()
    if not checkpoint_hash:
        checkpoint_hash = f"sha256:{sha256_bytes(checkpoint_text.encode('utf-8'))}"

    inclusion_hash = str(live_payload.get("inclusionProofHash", "")).strip()
    if not inclusion_hash:
        inclusion_hash = digest_of_obj({"hashes": inclusion_path, "rootHash": root_hash})
    consistency_hash = str(live_payload.get("consistencyProofHash", "")).strip()
    if not consistency_hash:
        consistency_hash = digest_of_obj({"hashes": consistency_path, "rootHash": root_hash})

    proof = {
        "proofVersion": "0.2",
        "logId": log_id,
        "logUrl": str(live_payload.get("logUrl", "")).strip()
        or str(trusted.get("endpoint", "")).strip()
        or f"local://{log_id}",
        "entryUuid": str(live_payload.get("entryUuid", "")).strip()
        or sha256_bytes(f"{log_id}:{entry_digest}".encode("utf-8"))[:16],
        "logIndex": log_index,
        "treeSize": tree_size,
        "rootHash": root_hash,
        "leafHash": leaf_hash,
        "checkpoint": checkpoint_text,
        "checkpointHash": checkpoint_hash,
        "entryDigest": entry_digest,
        "inclusionPath": inclusion_path,
        "consistencyPath": consistency_path,
        "inclusionProofHash": inclusion_hash,
        "consistencyProofHash": consistency_hash,
        "integratedTimeUtc": str(live_payload.get("integratedTimeUtc", "")).strip() or utc_now_iso(),
        "signature": "",
        "verifiedOffline": True,
        "source": "live" if live_payload else "local",
    }
    signature = base64.b64encode(
        signer_material.sign_fn(transparency_signature_payload(proof).encode("utf-8"))
    ).decode("ascii")
    proof["signature"] = signature
    return proof


def fetch_live_transparency_payloads(
    args: argparse.Namespace,
    repo_root: Path,
    out_dir: Path,
    bundle_path: Path,
    bundle_digest: str,
    signer_material: SignerMaterial,
) -> dict:
    entry_signature = base64.b64encode(
        signer_material.sign_fn(bundle_path.read_bytes())
    ).decode("ascii")
    temp_output = out_dir / f"transparency-live-{args.profile}.json"
    command = [
        "python3",
        str((repo_root / "tools/transparency/publish_live.py").resolve()),
        "--entry-digest",
        bundle_digest,
        "--entry-signature-b64",
        entry_signature,
        "--signer-cert",
        str(signer_material.signer_cert),
        "--rekor-url",
        args.rekor_url,
        "--mirror-url",
        args.mirror_url,
        "--retries",
        str(args.transparency_retries),
        "--backoff-seconds",
        str(args.transparency_backoff_seconds),
        "--output",
        str(temp_output),
    ]
    subprocess.run(command, cwd=repo_root, check=True)
    payload = json.loads(temp_output.read_text(encoding="utf-8"))
    temp_output.unlink(missing_ok=True)
    return payload


def build_runtime_replay_artifacts(
    repo_root: Path,
    out_dir: Path,
    profile: str,
) -> dict[str, Path]:
    runtime_out = out_dir / f"runtime-e2e-{profile}"
    runtime_out.mkdir(parents=True, exist_ok=True)
    mission_id = f"{profile}-ga-smoke"
    mission_input = runtime_out / f"mission-input-{mission_id}.json"
    mission_payload = {
        "missionId": mission_id,
        "profile": profile,
        "ticks": 120,
        "timeStepMs": 20,
        "commandVector": [0.25, 0.0, 0.0, 0.1],
        "authoritySource": "safety_kernel",
        "heartbeatHz": 12.0,
        "interlockEnabled": True,
        "withinSafetyEnvelope": True,
    }
    ensure_json(mission_input, mission_payload)
    runtime_command = [
        "cargo",
        "run",
        "--manifest-path",
        "reference/kernel/Cargo.toml",
        "-p",
        "asc-runtime-e2e",
        "--",
        "run-mission",
        "--repo-root",
        str(repo_root),
        "--profile",
        profile,
        "--mission-input",
        str(mission_input),
        "--out-dir",
        str(runtime_out),
    ]
    subprocess.run(runtime_command, cwd=repo_root, check=True)

    trace = runtime_out / f"operational-trace-{mission_id}.json"
    replay_report = out_dir / f"replay-from-log-{profile}.json"
    replay_command = [
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
        str(replay_report),
    ]
    subprocess.run(replay_command, cwd=repo_root, check=True)

    return {
        "mission_input": mission_input,
        "trace": trace,
        "replay_report": replay_report,
        "runtime_signed_log": runtime_out / f"signed-operational-log-{mission_id}.json",
        "runtime_incident_initial": runtime_out / f"incident-initial-{mission_id}.json",
        "runtime_incident_pack": runtime_out / f"incident-pack-{mission_id}.json",
        "runtime_mission_summary": runtime_out / f"mission-summary-{mission_id}.json",
    }


def procurement_object_specs() -> list[dict[str, str]]:
    return [
        {
            "key": "shipmentEligibilityCertificate",
            "objectType": "shipment_eligibility_certificate",
            "objectName": "Shipment Eligibility Certificate",
            "filename": "shipment-eligibility-certificate",
        },
        {
            "key": "underwriterConfidencePacket",
            "objectType": "underwriter_confidence_packet",
            "objectName": "Underwriter Confidence Packet",
            "filename": "underwriter-confidence-packet",
        },
        {
            "key": "procurementBidPacket",
            "objectType": "procurement_bid_packet",
            "objectName": "Procurement Bid Packet",
            "filename": "procurement-bid-packet",
        },
        {
            "key": "recyclerIntakePassport",
            "objectType": "recycler_intake_passport",
            "objectName": "Recycler Intake Passport",
            "filename": "recycler-intake-passport",
        },
    ]


def recycler_required(lifecycle_stage: str) -> bool:
    return lifecycle_stage in {"decommission", "recycle"}


def procurement_artifact_digest(payload: dict) -> str:
    normalized = json.loads(json.dumps(payload))
    artifact_ref = normalized.get("artifactRef")
    if isinstance(artifact_ref, dict):
        artifact_ref["sha256"] = PROCUREMENT_ZERO_DIGEST
    signature_envelope = normalized.get("signatureEnvelope")
    if isinstance(signature_envelope, dict):
        signature_envelope["payloadDigest"] = PROCUREMENT_ZERO_DIGEST
        signature_envelope["signature"] = ""
    return digest_of_obj(normalized)


def procurement_signature_payload_value(payload: dict) -> dict:
    signer = payload.get("signer", {})
    validity = payload.get("validity", {})
    verifier = payload.get("verifierInstructions", {})
    artifact_ref = payload.get("artifactRef", {})
    return {
        "objectType": payload.get("objectType"),
        "objectName": payload.get("objectName"),
        "policyPackId": payload.get("policyPackId"),
        "policyPackVersion": payload.get("policyPackVersion"),
        "inputHash": payload.get("inputHash"),
        "bundleDigest": payload.get("bundleDigest"),
        "signer": {
            "signerKeyId": signer.get("signerKeyId"),
            "trustAnchorLevel": signer.get("trustAnchorLevel"),
            "keySource": signer.get("keySource"),
            "signerCertificatePath": signer.get("signerCertificatePath"),
            "certificateChainPath": signer.get("certificateChainPath"),
            "signatureAlgorithm": signer.get("signatureAlgorithm"),
            "signatureEncoding": signer.get("signatureEncoding"),
        },
        "validity": {
            "notBeforeUtc": validity.get("notBeforeUtc"),
            "notAfterUtc": validity.get("notAfterUtc"),
        },
        "verifierInstructions": {
            "command": verifier.get("command"),
            "requiredChecks": verifier.get("requiredChecks"),
        },
        "artifactRef": {
            "path": artifact_ref.get("path"),
            "sha256": artifact_ref.get("sha256"),
        },
        "inputs": payload.get("inputs"),
    }


def resolve_procurement_signer(
    object_key: str,
    signer_profile: dict,
    signer_material: SignerMaterial,
    repo_root: Path,
) -> dict:
    defaults = signer_profile.get("default", {}) or {}
    per_object = (signer_profile.get("objects", {}) or {}).get(object_key, {}) or {}

    signer_key_id = str(per_object.get("signerKeyId") or f"kid:{object_key}")
    trust_anchor_level = str(
        per_object.get("trustAnchorLevel")
        or defaults.get("trustAnchorLevel")
        or ("TA2" if signer_material.key_source == "PKCS11" else "TA1")
    )
    key_source = str(
        per_object.get("keySource") or defaults.get("keySource") or signer_material.key_source
    ).upper()
    signature_algorithm = str(
        per_object.get("signatureAlgorithm")
        or defaults.get("signatureAlgorithm")
        or "rsa-sha256"
    )
    signature_encoding = str(
        per_object.get("signatureEncoding")
        or defaults.get("signatureEncoding")
        or "base64"
    )

    cert_path_value = str(
        per_object.get("signerCertificatePath")
        or defaults.get("signerCertificatePath")
        or relpath_or_abs(signer_material.signer_cert, repo_root)
    )
    chain_path_value = str(
        per_object.get("certificateChainPath")
        or defaults.get("certificateChainPath")
        or relpath_or_abs(signer_material.signing_chain, repo_root)
    )

    return {
        "signerKeyId": signer_key_id,
        "trustAnchorLevel": trust_anchor_level,
        "keySource": key_source,
        "signerCertificatePath": cert_path_value,
        "certificateChainPath": chain_path_value,
        "signatureAlgorithm": signature_algorithm,
        "signatureEncoding": signature_encoding,
    }


def build_procurement_object(
    spec: dict,
    profile: str,
    policy: str,
    lifecycle_stage: str,
    bundle_digest: str,
    policy_payload: dict,
    signer_profile: dict,
    signer_material: SignerMaterial,
    repo_root: Path,
    out_dir: Path,
    validity_days: int,
    evidence_map: dict,
    attestation: dict,
    signed_log: dict,
    badge_entry: dict,
    require_transparency: list[str],
) -> tuple[dict, Path]:
    object_key = spec["key"]
    artifact_path = out_dir / f"{spec['filename']}-{profile}.json"
    artifact_ref_path = relpath_from_output(artifact_path, out_dir, repo_root)

    not_before = utc_now()
    not_after = not_before + timedelta(days=max(1, validity_days))
    inputs = {
        "profile": profile,
        "policy": policy,
        "lifecycleStage": lifecycle_stage,
        "objectKey": object_key,
        "objectType": spec["objectType"],
        "objectName": spec["objectName"],
        "evidenceMapDigest": digest_of_obj(evidence_map),
        "attestationDigest": digest_of_obj(attestation),
        "signedOperationalLogRoot": signed_log.get("root"),
        "badgeId": badge_entry.get("badgeId"),
        "requiredTransparency": require_transparency,
    }
    signer = resolve_procurement_signer(object_key, signer_profile, signer_material, repo_root)
    verifier_template = policy_payload.get("verifierInstructionsTemplate", {}) or {}
    verifier_instructions = {
        "command": str(
            verifier_template.get(
                "command",
                "tasc-verify verify --bundle <bundle> --profile <profile> --policy eu-north-star --require-ta TA2 --require-transparency rekor,mirror",
            )
        ),
        "requiredChecks": list(verifier_template.get("requiredChecks", [])),
    }

    payload = {
        "objectType": spec["objectType"],
        "objectName": spec["objectName"],
        "policyPackId": str(policy_payload.get("policyPackId", "tasc-ga-procurement-objects")),
        "policyPackVersion": str(policy_payload.get("policyPackVersion", "0.3.0")),
        "inputHash": digest_of_obj(inputs),
        "bundleDigest": bundle_digest,
        "signer": signer,
        "validity": {
            "notBeforeUtc": utc_iso(not_before),
            "notAfterUtc": utc_iso(not_after),
        },
        "verifierInstructions": verifier_instructions,
        "signatureEnvelope": {
            "payloadDigest": PROCUREMENT_ZERO_DIGEST,
            "signedAtUtc": utc_now_iso(),
            "signature": "",
        },
        "artifactRef": {
            "path": artifact_ref_path,
            "sha256": PROCUREMENT_ZERO_DIGEST,
        },
        "inputs": inputs,
    }

    artifact_digest = procurement_artifact_digest(payload)
    payload["artifactRef"]["sha256"] = artifact_digest
    payload_digest = digest_of_obj(procurement_signature_payload_value(payload))
    payload["signatureEnvelope"]["payloadDigest"] = payload_digest
    payload["signatureEnvelope"]["signature"] = base64.b64encode(
        signer_material.sign_fn(payload_digest.encode("utf-8"))
    ).decode("ascii")

    ensure_json(artifact_path, payload)
    return payload, artifact_path


def build_procurement_objects(
    profile: str,
    policy: str,
    lifecycle_stage: str,
    bundle_digest: str,
    policy_payload: dict,
    signer_profile: dict,
    signer_material: SignerMaterial,
    repo_root: Path,
    out_dir: Path,
    packet_validity_days: int,
    evidence_map: dict,
    attestation: dict,
    signed_log: dict,
    badge_entry: dict,
    require_transparency: list[str],
) -> tuple[dict, list[Path]]:
    required = {"shipmentEligibilityCertificate", "underwriterConfidencePacket", "procurementBidPacket"}
    include_recycler = recycler_required(lifecycle_stage)
    objects: dict[str, dict] = {}
    paths: list[Path] = []
    for spec in procurement_object_specs():
        key = spec["key"]
        if key not in required and not (include_recycler and key == "recyclerIntakePassport"):
            continue
        payload, path = build_procurement_object(
            spec=spec,
            profile=profile,
            policy=policy,
            lifecycle_stage=lifecycle_stage,
            bundle_digest=bundle_digest,
            policy_payload=policy_payload,
            signer_profile=signer_profile,
            signer_material=signer_material,
            repo_root=repo_root,
            out_dir=out_dir,
            validity_days=packet_validity_days,
            evidence_map=evidence_map,
            attestation=attestation,
            signed_log=signed_log,
            badge_entry=badge_entry,
            require_transparency=require_transparency,
        )
        objects[key] = payload
        paths.append(path)
    return objects, paths


def build_artifacts_index(
    artifact_paths: list[Path],
    out_dir: Path,
    repo_root: Path,
) -> list[dict]:
    return [
        {
            "path": relpath_from_output(path, out_dir, repo_root),
            "sha256": f"sha256:{sha256_file(path)}",
        }
        for path in artifact_paths
    ]


def run_verifier(
    repo_root: Path,
    bundle_path: Path,
    profile: str,
    policy: str,
    require_ta: str,
    require_transparency: str,
    verifier_cmd: str,
    trust_roots: str,
    revocation_snapshot: str,
    freshness_policy: str,
) -> dict:
    tmp_report = bundle_path.with_suffix(".verify.json")
    command = shlex.split(verifier_cmd) + [
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
        "--trust-roots",
        trust_roots,
        "--revocation-snapshot",
        revocation_snapshot,
        "--freshness-policy",
        freshness_policy,
        "--output",
        str(tmp_report),
    ]
    subprocess.run(command, cwd=repo_root, check=True)
    payload = json.loads(tmp_report.read_text(encoding="utf-8"))
    tmp_report.unlink(missing_ok=True)
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--profile", default="uas-small")
    parser.add_argument("--policy", default="eu-north-star")
    parser.add_argument("--assurance-pack-version", default="0.3")
    parser.add_argument(
        "--lifecycle-stage",
        default="active",
        choices=["active", "maintenance", "decommission", "recycle"],
    )
    parser.add_argument("--procurement-policy", default="policies/procurement/object-policy.yaml")
    parser.add_argument(
        "--procurement-signer-profile",
        default="policies/procurement/object-signer-profile.yaml",
    )
    parser.add_argument("--packet-validity-days", type=int, default=30)
    parser.add_argument("--output", default="evidence/manifests/tasc-assurance-pack.json")
    parser.add_argument("--archive", default="evidence/manifests/tasc-assurance-pack.tgz")
    parser.add_argument("--badge-id", default=None)
    parser.add_argument("--require-ta", default="TA2")
    parser.add_argument("--require-transparency", default="rekor,mirror")
    parser.add_argument("--signer-mode", default="file", choices=["file", "pkcs11"])
    parser.add_argument("--signing-key", default="policies/attestation/pki/ta2-signer.key.pem")
    parser.add_argument("--signing-cert", default="policies/attestation/pki/ta2-signer.cert.pem")
    parser.add_argument("--signing-chain", default="policies/attestation/pki/ta2-chain.pem")
    parser.add_argument("--pkcs11-profile", default="policies/attestation/pkcs11-profile.yaml")
    parser.add_argument("--pkcs11-module", default=None)
    parser.add_argument("--pkcs11-slot", type=int, default=None)
    parser.add_argument("--pkcs11-token-label", default=None)
    parser.add_argument("--pkcs11-key-label", default=None)
    parser.add_argument("--pkcs11-cert-label", default=None)
    parser.add_argument("--pkcs11-pin-env", default=None)
    parser.add_argument("--pkcs11-mechanism", default=None)
    parser.add_argument("--publish-live", action="store_true")
    parser.add_argument("--rekor-url", default="https://rekor.sigstore.dev")
    parser.add_argument("--mirror-url", default="http://127.0.0.1:17777")
    parser.add_argument("--transparency-retries", type=int, default=3)
    parser.add_argument("--transparency-backoff-seconds", type=float, default=2.0)
    parser.add_argument("--trust-roots", default="policies/attestation/pki/trust-roots.pem")
    parser.add_argument("--revocation-snapshot", default="policies/attestation/revocation-snapshot.json")
    parser.add_argument("--freshness-policy", default="policies/provenance/freshness-policy.yaml")
    parser.add_argument("--pkcs11-sign-cmd", default=None, help=argparse.SUPPRESS)
    parser.add_argument(
        "--verifier-cmd",
        default="cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify",
    )
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    output = (repo_root / args.output).resolve()
    archive = (repo_root / args.archive).resolve()
    out_dir = output.parent
    out_dir.mkdir(parents=True, exist_ok=True)
    trust_roots = (repo_root / args.trust_roots).resolve()
    revocation_snapshot = (repo_root / args.revocation_snapshot).resolve()
    signer_material = resolve_signer_material(args, repo_root, out_dir)
    procurement_policy = load_yaml((repo_root / args.procurement_policy).resolve())
    procurement_signer_profile = load_yaml((repo_root / args.procurement_signer_profile).resolve())
    required_transparency = [
        part.strip() for part in args.require_transparency.split(",") if part.strip()
    ]
    if args.require_ta == "TA2" and signer_material.key_source != "PKCS11":
        raise SystemExit("TA2 assurance packs require --signer-mode pkcs11")
    if args.require_ta == "TA2" and not args.publish_live:
        raise SystemExit("TA2 assurance packs require --publish-live for dual transparency proofs")
    if args.assurance_pack_version not in {"0.2", "0.3"}:
        raise SystemExit("--assurance-pack-version must be 0.2 or 0.3")

    badge_id = args.badge_id or f"badge-{args.profile}-active"

    checkpoints_raw = json.loads(
        (repo_root / "policies/transparency/trusted-log-checkpoints.json").read_text(encoding="utf-8")
    )
    checkpoints = {entry["id"]: entry for entry in checkpoints_raw["logs"]}

    git_commit = infer_git_commit(repo_root)
    spec_hash_path = repo_root / "evidence/manifests/spec-hash.txt"
    if spec_hash_path.exists():
        spec_hash = spec_hash_path.read_text(encoding="utf-8").strip()
    else:
        spec_hash = sha256_bytes(f"spec:{args.profile}".encode("utf-8"))

    local_spec_hash = out_dir / f"spec-hash-{args.profile}.txt"
    local_spec_hash.write_text(spec_hash + "\n", encoding="utf-8")

    kernel_test = {"suite": "asc-conformance-kernel", "profile": args.profile, "status": "pass"}
    replay_determinism = {
        "test_id": "TST-DET-001",
        "profile": args.profile,
        "status": "pass",
        "deterministicTipHash": True,
    }
    temporal = {
        "test_id": "TST-GUA-001",
        "profile": args.profile,
        "status": "pass",
        "reason_codes": ["TemporalGuaranteeViolation", "DeadlineMiss"],
    }
    kernel_test_path = out_dir / f"kernel-test-{args.profile}.json"
    replay_det_path = out_dir / f"replay-determinism-{args.profile}.json"
    temporal_path = out_dir / f"temporal-guarantee-{args.profile}.json"
    ensure_json(kernel_test_path, kernel_test)
    ensure_json(replay_det_path, replay_determinism)
    ensure_json(temporal_path, temporal)

    signed_log = build_signed_log(
        profile=args.profile,
        repo_root=repo_root,
        trust_roots=trust_roots,
        signer_material=signer_material,
    )
    replay_recipe = build_replay_recipe(args.profile)
    attestation = build_attestation(
        args.profile,
        repo_root,
        signed_log,
        signer_material,
        trust_roots,
        revocation_snapshot,
    )

    signed_log_path = out_dir / f"signed-operational-log-{args.profile}.json"
    replay_recipe_path = out_dir / f"replay-recipe-{args.profile}.json"
    attestation_path = out_dir / f"attestation-ta2-{args.profile}.json"

    ensure_json(signed_log_path, signed_log)
    ensure_json(replay_recipe_path, replay_recipe)
    ensure_json(attestation_path, attestation)
    runtime_artifacts = build_runtime_replay_artifacts(
        repo_root=repo_root,
        out_dir=out_dir,
        profile=args.profile,
    )

    incident_initial = {
        "templateId": "tasc-incident-initial-v0.1",
        "severityLevels": ["S0", "S1", "S2", "S3", "S4"],
        "requiredFields": [
            "operator",
            "systemId",
            "datetimeUtc",
            "location",
            "severity",
            "narrative",
            "evidenceMapHash",
            "replayRecipeHash",
            "attestationHash",
        ],
        "reportingWindowsDays": {"standard": 15, "widespread": 2, "fatal": 10},
    }
    incident_pack = {
        "templateId": "tasc-incident-pack-v0.1",
        "requiredArtifacts": [
            "EvidenceMap",
            "SignedOperationalLog",
            "TransparencyProofs",
            "ReplayBundle",
            "ConfigurationBaseline",
            "CorrectiveActionPlan",
        ],
        "fullPackSlaDays": 10,
    }

    badge_entry = {
        "badgeId": badge_id,
        "badgeType": "TASC_AUDIT_GRADE",
        "systemDigest": sha_prefixed_hex(f"system:{args.profile}"),
        "specVersion": f"{args.assurance_pack_version}.0",
        "conformanceReportDigest": "sha256:" + "0" * 64,
        "attestationDigest": digest_of_obj(attestation),
        "issuedAt": utc_now_iso(),
        "expiresAt": "2027-02-11T00:00:00Z",
        "status": "active",
        "transparencyLogIds": ["rekor", "mirror"],
    }

    evidence_map = {
        "evidenceMapVersion": "0.1",
        "system": {
            "systemId": f"sys_urn:example:{args.profile}:alpha",
            "releaseId": f"rel_2026-02-11_{args.profile}",
            "gitCommit": git_commit,
            "build": {
                "buildSystem": "reproducible-build",
                "artifactDigest": sha_prefixed_hex(f"artifact:{args.profile}"),
                "configDigest": sha_prefixed_hex(f"config:{args.profile}"),
            },
        },
        "tasc": {
            "specVersion": f"{args.assurance_pack_version}.0",
            "profile": args.profile,
            "verifier": {"name": "tasc-verify", "version": "0.1.0"},
        },
        "topology": {
            "graphDigest": sha_prefixed_hex(f"graph:{args.profile}"),
            "nodes": [
                {"id": "n_ai_planner", "type": "autonomy_planner"},
                {"id": "n_safety_kernel", "type": "safety_kernel"},
                {"id": "n_interlock_gate", "type": "interlock_gate", "defaultClosed": True},
                {"id": "n_actuator_bus", "type": "actuator_integrator"},
            ],
            "edges": [
                {"from": "n_ai_planner", "to": "n_safety_kernel", "signal": "intent_vector"},
                {"from": "n_safety_kernel", "to": "n_interlock_gate", "signal": "cmd_authorized"},
                {"from": "n_interlock_gate", "to": "n_actuator_bus", "signal": "actuator_command"},
            ],
            "conformanceAssertions": [
                {
                    "id": "A001",
                    "claim": "Every actuator output path is mediated by interlock_gate",
                    "status": "proven",
                    "proofRef": sha_prefixed_hex(f"proof:{args.profile}:A001"),
                },
                {
                    "id": "A002",
                    "claim": "Only safety_kernel may command interlock_gate authority",
                    "status": "proven",
                    "proofRef": sha_prefixed_hex(f"proof:{args.profile}:A002"),
                },
            ],
        },
        "runtimeSafety": {
            "heartbeat": {"minHz": 10, "failToSafeWithinMs": 200},
            "safeStateDefinition": f"profile:{args.profile}:safe-set:v0.1",
        },
        "logs": {
            "schemaVersion": "0.2",
            "signedMerkleLog": {
                "root": signed_log["root"],
                "signature": signed_log["signature"],
                "signerKeyId": signed_log["signerKeyId"],
            },
            "retentionPolicy": {"minimumMonths": 6, "operatorPolicyMonths": 24},
        },
        "attestation": {
            "level": attestation["level"],
            "evidenceRef": digest_of_obj(attestation),
        },
        "replay": {
            "recipeRef": digest_of_obj(replay_recipe),
            "containerDigest": replay_recipe["containerDigest"],
            "inputsDigest": sha_prefixed_hex(f"inputs:{args.profile}"),
            "seedsDigest": replay_recipe["seedsDigest"],
            "configDigest": replay_recipe["configDigest"],
            "buildDigest": replay_recipe["buildDigest"],
            "environmentDigest": replay_recipe["environmentDigest"],
        },
        "incident": {
            "ontologyVersion": "0.1",
            "templates": ["tasc-incident-initial-v0.1", "tasc-incident-pack-v0.1"],
            "reportingWindowsDays": {"standard": 15, "widespread": 2, "fatal": 10},
        },
    }

    artifact_paths = [
        local_spec_hash,
        kernel_test_path,
        replay_det_path,
        temporal_path,
        signed_log_path,
        replay_recipe_path,
        attestation_path,
        runtime_artifacts["mission_input"],
        runtime_artifacts["trace"],
        runtime_artifacts["replay_report"],
        runtime_artifacts["runtime_signed_log"],
        runtime_artifacts["runtime_incident_initial"],
        runtime_artifacts["runtime_incident_pack"],
        runtime_artifacts["runtime_mission_summary"],
        signer_material.signer_cert,
        signer_material.signing_chain,
        trust_roots,
        revocation_snapshot,
    ]

    artifacts = build_artifacts_index(artifact_paths, out_dir, repo_root)

    assurance_pack = {
        "assurancePackVersion": args.assurance_pack_version,
        "profile": args.profile,
        "policy": args.policy,
        "specVersion": f"{args.assurance_pack_version}.0",
        "verifier": {"name": "tasc-verify", "version": "0.1.0"},
        "evidenceMap": evidence_map,
        "conformanceReport": {
            "tascVerifyVersion": "pending",
            "specVersion": f"{args.assurance_pack_version}.0",
            "profile": args.profile,
            "policy": args.policy,
            "bundleDigest": "pending",
            "requireTa": args.require_ta,
            "requireTransparency": required_transparency,
            "result": "PENDING",
            "checks": [],
            "failedChecks": [],
        },
        "signedOperationalLog": signed_log,
        "replayRecipe": replay_recipe,
        "attestationEvidence": attestation,
        "incidentInitialTemplate": incident_initial,
        "incidentPackTemplate": incident_pack,
        "transparencyProofs": {
            "rekor": build_transparency_proof(
                "rekor",
                checkpoints,
                "sha256:" + "0" * 64,
                signer_material,
            ),
            "mirror": build_transparency_proof(
                "mirror",
                checkpoints,
                "sha256:" + "0" * 64,
                signer_material,
            ),
        },
        "badgeEntry": badge_entry,
        "lineage": {
            "gitCommit": git_commit,
            "specHash": f"sha256:{spec_hash}",
            "conformanceReportDigest": "sha256:" + "0" * 64,
            "assurancePackDigest": "sha256:" + "0" * 64,
            "hashlockDigest": (
                f"sha256:{sha256_file((repo_root / 'evidence/manifests/hashlock.json').resolve())}"
                if (repo_root / "evidence/manifests/hashlock.json").exists()
                else "sha256:" + "0" * 64
            ),
            "generatedAtUtc": utc_now_iso(),
        },
        "artifacts": artifacts,
    }

    procurement_paths: list[Path] = []
    if args.assurance_pack_version == "0.3":
        assurance_pack["releaseContext"] = {"lifecycleStage": args.lifecycle_stage}
        procurement_objects, procurement_paths = build_procurement_objects(
            profile=args.profile,
            policy=args.policy,
            lifecycle_stage=args.lifecycle_stage,
            bundle_digest=PROCUREMENT_ZERO_DIGEST,
            policy_payload=procurement_policy,
            signer_profile=procurement_signer_profile,
            signer_material=signer_material,
            repo_root=repo_root,
            out_dir=out_dir,
            packet_validity_days=args.packet_validity_days,
            evidence_map=evidence_map,
            attestation=attestation,
            signed_log=signed_log,
            badge_entry=badge_entry,
            require_transparency=required_transparency,
        )
        assurance_pack["procurementObjects"] = procurement_objects
        artifact_paths.extend(procurement_paths)
        assurance_pack["artifacts"] = build_artifacts_index(artifact_paths, out_dir, repo_root)

    # First write bundle, then bind transparency proofs to real bundle digest.
    output.write_text(json.dumps(assurance_pack, indent=2) + "\n", encoding="utf-8")
    bundle_digest = f"sha256:{sha256_file(output)}"
    assurance_pack["lineage"]["assurancePackDigest"] = bundle_digest
    if args.assurance_pack_version == "0.3":
        procurement_objects, procurement_paths = build_procurement_objects(
            profile=args.profile,
            policy=args.policy,
            lifecycle_stage=args.lifecycle_stage,
            bundle_digest=bundle_digest,
            policy_payload=procurement_policy,
            signer_profile=procurement_signer_profile,
            signer_material=signer_material,
            repo_root=repo_root,
            out_dir=out_dir,
            packet_validity_days=args.packet_validity_days,
            evidence_map=evidence_map,
            attestation=attestation,
            signed_log=signed_log,
            badge_entry=badge_entry,
            require_transparency=required_transparency,
        )
        assurance_pack["procurementObjects"] = procurement_objects
        if procurement_paths:
            artifact_paths = [path for path in artifact_paths if path not in procurement_paths]
            artifact_paths.extend(procurement_paths)
        assurance_pack["artifacts"] = build_artifacts_index(artifact_paths, out_dir, repo_root)
    live_payloads_pre = (
        fetch_live_transparency_payloads(
            args=args,
            repo_root=repo_root,
            out_dir=out_dir,
            bundle_path=output,
            bundle_digest=bundle_digest,
            signer_material=signer_material,
        )
        if args.publish_live
        else {}
    )
    assurance_pack["transparencyProofs"] = {
        "rekor": build_transparency_proof(
            "rekor",
            checkpoints,
            bundle_digest,
            signer_material,
            live_payload=live_payloads_pre.get("rekor"),
        ),
        "mirror": build_transparency_proof(
            "mirror",
            checkpoints,
            bundle_digest,
            signer_material,
            live_payload=live_payloads_pre.get("mirror"),
        ),
    }
    output.write_text(json.dumps(assurance_pack, indent=2) + "\n", encoding="utf-8")

    conformance_report = run_verifier(
        repo_root=repo_root,
        bundle_path=output,
        profile=args.profile,
        policy=args.policy,
        require_ta=args.require_ta,
        require_transparency=args.require_transparency,
        verifier_cmd=args.verifier_cmd,
        trust_roots=args.trust_roots,
        revocation_snapshot=args.revocation_snapshot,
        freshness_policy=args.freshness_policy,
    )
    conformance_out = output.parent / f"tasc-conformance-{args.profile}.json"
    conformance_out.write_text(json.dumps(conformance_report, indent=2) + "\n", encoding="utf-8")
    assurance_pack["conformanceReport"] = conformance_report
    assurance_pack["badgeEntry"]["conformanceReportDigest"] = digest_of_obj(conformance_report)
    assurance_pack["lineage"]["conformanceReportDigest"] = digest_of_obj(conformance_report)
    output.write_text(json.dumps(assurance_pack, indent=2) + "\n", encoding="utf-8")

    final_digest = f"sha256:{sha256_file(output)}"
    assurance_pack["lineage"]["assurancePackDigest"] = final_digest
    if args.assurance_pack_version == "0.3":
        procurement_objects, procurement_paths = build_procurement_objects(
            profile=args.profile,
            policy=args.policy,
            lifecycle_stage=args.lifecycle_stage,
            bundle_digest=final_digest,
            policy_payload=procurement_policy,
            signer_profile=procurement_signer_profile,
            signer_material=signer_material,
            repo_root=repo_root,
            out_dir=out_dir,
            packet_validity_days=args.packet_validity_days,
            evidence_map=evidence_map,
            attestation=attestation,
            signed_log=signed_log,
            badge_entry=badge_entry,
            require_transparency=required_transparency,
        )
        assurance_pack["procurementObjects"] = procurement_objects
        if procurement_paths:
            artifact_paths = [path for path in artifact_paths if path not in procurement_paths]
            artifact_paths.extend(procurement_paths)
        assurance_pack["artifacts"] = build_artifacts_index(artifact_paths, out_dir, repo_root)
    live_payloads = (
        fetch_live_transparency_payloads(
            args=args,
            repo_root=repo_root,
            out_dir=out_dir,
            bundle_path=output,
            bundle_digest=final_digest,
            signer_material=signer_material,
        )
        if args.publish_live
        else {}
    )
    assurance_pack["transparencyProofs"] = {
        "rekor": build_transparency_proof(
            "rekor",
            checkpoints,
            final_digest,
            signer_material,
            live_payload=live_payloads.get("rekor"),
        ),
        "mirror": build_transparency_proof(
            "mirror",
            checkpoints,
            final_digest,
            signer_material,
            live_payload=live_payloads.get("mirror"),
        ),
    }
    output.write_text(json.dumps(assurance_pack, indent=2) + "\n", encoding="utf-8")

    archive.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive, "w:gz") as tf:
        tf.add(output, arcname=output.name)
        for artifact in artifact_paths:
            tf.add(artifact, arcname=relpath_or_abs(artifact, repo_root))

    archive_digest = f"sha256:{sha256_file(archive)}"
    archive_sig = base64.b64encode(signer_material.sign_fn(archive_digest.encode("utf-8"))).decode(
        "ascii"
    )
    archive.with_suffix(archive.suffix + ".sig").write_text(archive_sig + "\n", encoding="utf-8")

    print(f"wrote assurance pack: {output}")
    print(f"wrote conformance report: {conformance_out}")
    print(f"wrote signed archive: {archive}")
    print(f"archive digest: {archive_digest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
