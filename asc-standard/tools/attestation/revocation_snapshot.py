#!/usr/bin/env python3
"""Build and sign revocation-snapshot.json from CRL and OCSP source policy."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import subprocess
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path
from tempfile import NamedTemporaryFile
from typing import Any
from urllib import request

import yaml


def utc_now() -> datetime:
    return datetime.now(timezone.utc).replace(microsecond=0)


def utc_iso(value: datetime) -> str:
    return value.isoformat().replace("+00:00", "Z")


def sha_prefixed(data: bytes) -> str:
    return f"sha256:{hashlib.sha256(data).hexdigest()}"


def canonical_json(payload: object) -> bytes:
    return json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")


def openssl_sign(private_key: Path, payload: bytes) -> bytes:
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
            capture_output=True,
            text=True,
        )
        return out_path.read_bytes()
    finally:
        in_path.unlink(missing_ok=True)
        out_path.unlink(missing_ok=True)


def cert_serial(cert_path: Path) -> str:
    proc = subprocess.run(
        ["openssl", "x509", "-in", str(cert_path), "-serial", "-noout"],
        check=True,
        capture_output=True,
        text=True,
    )
    raw = proc.stdout.strip()
    return raw.split("=", 1)[1].strip().upper() if "=" in raw else raw.upper()


def parse_crl_revoked(crl_path: Path) -> list[str]:
    proc = subprocess.run(
        ["openssl", "crl", "-in", str(crl_path), "-text", "-noout"],
        check=True,
        capture_output=True,
        text=True,
    )
    revoked: list[str] = []
    for line in proc.stdout.splitlines():
        line = line.strip()
        if line.startswith("Serial Number:"):
            revoked.append(line.split(":", 1)[1].strip().upper())
    return revoked


def load_bytes_from_source(path: str | None = None, url: str | None = None) -> bytes:
    if path:
        return Path(path).read_bytes()
    if url:
        with request.urlopen(url, timeout=15) as resp:
            return resp.read()
    raise RuntimeError("source must provide path or url")


@dataclass
class CertSource:
    cert_path: Path
    issuer_path: Path | None
    cert_id: str


def load_sources(path: Path) -> dict[str, Any]:
    payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise RuntimeError("revocation source policy must be a mapping")
    return payload


def ocsp_statuses_from_policy(
    policy: dict[str, Any],
    certs: list[CertSource],
    now_iso: str,
) -> tuple[list[dict[str, Any]], dict[str, str]]:
    statuses: list[dict[str, Any]] = []
    source_digests: dict[str, str] = {}
    responders = policy.get("ocsp_responders", []) or []
    for responder in responders:
        source_id = str(responder.get("id", "")).strip()
        if not source_id:
            continue
        mode = str(responder.get("mode", "static")).strip()
        source_digests[source_id] = sha_prefixed(
            canonical_json(
                {
                    "id": source_id,
                    "mode": mode,
                    "status": responder.get("status", "unknown"),
                }
            )
        )
        if mode == "static":
            status_value = str(responder.get("status", "unknown")).strip().lower() or "unknown"
            for cert in certs:
                statuses.append(
                    {
                        "sourceId": source_id,
                        "certId": cert.cert_id,
                        "certSerial": cert_serial(cert.cert_path),
                        "status": status_value,
                        "checkedAtUtc": now_iso,
                        "detail": "static-policy",
                    }
                )
            continue

        url = str(responder.get("url", "")).strip()
        trust_roots = str(responder.get("trust_roots_path", "")).strip()
        for cert in certs:
            serial = cert_serial(cert.cert_path)
            if not cert.issuer_path or not url or not trust_roots:
                statuses.append(
                    {
                        "sourceId": source_id,
                        "certId": cert.cert_id,
                        "certSerial": serial,
                        "status": "unknown",
                        "checkedAtUtc": now_iso,
                        "detail": "missing ocsp url/issuer/trust roots",
                    }
                )
                continue
            command = [
                "openssl",
                "ocsp",
                "-issuer",
                str(cert.issuer_path),
                "-cert",
                str(cert.cert_path),
                "-url",
                url,
                "-CAfile",
                trust_roots,
                "-noverify",
                "-resp_text",
            ]
            proc = subprocess.run(command, capture_output=True, text=True, check=False)
            output = (proc.stdout or "") + "\n" + (proc.stderr or "")
            status = "unknown"
            if proc.returncode == 0:
                lowered = output.lower()
                if ": good" in lowered:
                    status = "good"
                elif ": revoked" in lowered:
                    status = "revoked"
            statuses.append(
                {
                    "sourceId": source_id,
                    "certId": cert.cert_id,
                    "certSerial": serial,
                    "status": status,
                    "checkedAtUtc": now_iso,
                    "detail": output.strip()[:500],
                }
            )
    return statuses, source_digests


def crl_revocation_from_policy(policy: dict[str, Any], repo_root: Path) -> tuple[list[str], dict[str, str]]:
    revoked: list[str] = []
    source_digests: dict[str, str] = {}
    for source in policy.get("crl_sources", []) or []:
        source_id = str(source.get("id", "")).strip()
        if not source_id:
            continue
        required = bool(source.get("required", True))
        path = source.get("path")
        url = source.get("url")
        resolved_path = str((repo_root / path).resolve()) if path else None
        try:
            raw = load_bytes_from_source(path=resolved_path, url=url)
        except Exception:
            if required:
                raise
            source_digests[source_id] = sha_prefixed(f"{source_id}:missing".encode("utf-8"))
            continue

        source_digests[source_id] = sha_prefixed(raw)
        with NamedTemporaryFile(delete=False) as tmp_file:
            tmp_path = Path(tmp_file.name)
            tmp_file.write(raw)
            tmp_file.flush()
        try:
            revoked.extend(parse_crl_revoked(tmp_path))
        finally:
            tmp_path.unlink(missing_ok=True)

    return sorted(set(revoked)), source_digests


def build_snapshot(repo_root: Path, policy_path: Path) -> dict[str, Any]:
    policy = load_sources(policy_path)
    now = utc_now()
    now_iso = utc_iso(now)
    rotation_hours = int(policy.get("rotation_hours", 24))
    next_update_iso = utc_iso(now + timedelta(hours=rotation_hours))

    certs: list[CertSource] = []
    observed: list[dict[str, str]] = []
    for item in policy.get("monitored_certificates", []) or []:
        cert_path = (repo_root / str(item.get("cert_path"))).resolve()
        issuer_value = item.get("issuer_path")
        issuer_path = (repo_root / str(issuer_value)).resolve() if issuer_value else None
        cert_id = str(item.get("id") or cert_path.name)
        certs.append(CertSource(cert_path=cert_path, issuer_path=issuer_path, cert_id=cert_id))
        subject_proc = subprocess.run(
            ["openssl", "x509", "-in", str(cert_path), "-subject", "-noout"],
            check=True,
            capture_output=True,
            text=True,
        )
        raw_subject = subject_proc.stdout.strip()
        subject = raw_subject.split("=", 1)[1].strip() if "=" in raw_subject else raw_subject
        observed.append(
            {
                "id": cert_id,
                "subject": subject,
                "serial": cert_serial(cert_path),
            }
        )

    revoked_from_crl, crl_source_digests = crl_revocation_from_policy(policy, repo_root)
    ocsp_statuses, ocsp_source_digests = ocsp_statuses_from_policy(policy, certs, now_iso)
    source_digests = dict(crl_source_digests)
    source_digests.update(ocsp_source_digests)

    payload = {
        "version": "0.3.0",
        "generatedAtUtc": now_iso,
        "nextUpdateUtc": next_update_iso,
        "revokedSerials": revoked_from_crl,
        "observedCertificates": observed,
        "sourceDigests": source_digests,
        "ocspStatuses": ocsp_statuses,
    }
    snapshot_id = sha_prefixed(canonical_json(payload))
    payload["snapshotId"] = snapshot_id
    return payload


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo-root", default=".")
    parser.add_argument(
        "--sources",
        default="policies/attestation/revocation-sources.yaml",
    )
    parser.add_argument(
        "--output",
        default="policies/attestation/revocation-snapshot.json",
    )
    parser.add_argument("--signing-key", default=None)
    parser.add_argument("--signing-cert", default=None)
    args = parser.parse_args()

    repo_root = Path(args.repo_root).resolve()
    sources_path = (repo_root / args.sources).resolve()
    output_path = (repo_root / args.output).resolve()
    policy = load_sources(sources_path)
    signing_policy = policy.get("signing", {}) or {}
    signing_key = (
        Path(args.signing_key).resolve()
        if args.signing_key
        else (repo_root / str(signing_policy.get("key_path", ""))).resolve()
    )
    signing_cert = (
        Path(args.signing_cert).resolve()
        if args.signing_cert
        else (repo_root / str(signing_policy.get("cert_path", ""))).resolve()
    )
    algorithm = str(signing_policy.get("algorithm", "rsa-sha256"))
    if algorithm != "rsa-sha256":
        raise SystemExit(f"unsupported revocation snapshot signing algorithm: {algorithm}")

    snapshot = build_snapshot(repo_root, sources_path)
    signature_payload = canonical_json(snapshot)
    signature_raw = openssl_sign(signing_key, signature_payload)
    snapshot["signature"] = base64.b64encode(signature_raw).decode("ascii")
    snapshot["signatureAlgorithm"] = algorithm
    snapshot["signerCertPath"] = str(signing_cert.relative_to(repo_root))

    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(snapshot, indent=2) + "\n", encoding="utf-8")
    print(f"wrote revocation snapshot: {output_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
