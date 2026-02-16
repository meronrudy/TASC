#!/usr/bin/env python3
"""Publish bundle digests to Rekor and mirror, then emit proof payloads."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import time
from datetime import datetime, timezone
from pathlib import Path
from typing import Any
from urllib import error, request


def utc_from_epoch(epoch: int | float) -> str:
    return datetime.fromtimestamp(float(epoch), tz=timezone.utc).replace(microsecond=0).isoformat().replace(
        "+00:00", "Z"
    )


def sha_prefixed_bytes(payload: bytes) -> str:
    return f"sha256:{hashlib.sha256(payload).hexdigest()}"


def sha_prefixed_text(text: str) -> str:
    return sha_prefixed_bytes(text.encode("utf-8"))


def json_hash(payload: object) -> str:
    canonical = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return sha_prefixed_bytes(canonical)


def http_json(
    *,
    method: str,
    url: str,
    payload: dict[str, Any] | None = None,
    timeout_s: int = 20,
) -> dict[str, Any]:
    data = None
    headers = {"Accept": "application/json"}
    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        headers["Content-Type"] = "application/json"
    req = request.Request(url=url, method=method, data=data, headers=headers)
    with request.urlopen(req, timeout=timeout_s) as resp:
        body = resp.read()
    return json.loads(body.decode("utf-8"))


def retry_http_json(
    *,
    method: str,
    url: str,
    payload: dict[str, Any] | None,
    retries: int,
    backoff_s: float,
) -> dict[str, Any]:
    attempt = 0
    last_err: Exception | None = None
    while attempt < retries:
        try:
            return http_json(method=method, url=url, payload=payload)
        except (error.HTTPError, error.URLError, TimeoutError, ValueError, json.JSONDecodeError) as exc:
            last_err = exc
            attempt += 1
            if attempt >= retries:
                break
            time.sleep(backoff_s * attempt)
    raise RuntimeError(f"{method} {url} failed after {retries} attempts: {last_err}")


def load_public_key_content(signer_cert_path: Path) -> str:
    import subprocess
    import tempfile

    with tempfile.NamedTemporaryFile(delete=False) as pub_file:
        pub_path = Path(pub_file.name)
    try:
        proc = subprocess.run(
            ["openssl", "x509", "-in", str(signer_cert_path), "-pubkey", "-noout"],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip()
            raise RuntimeError(f"openssl x509 -pubkey failed: {detail}")
        pubkey_pem = proc.stdout.encode("utf-8")
        return base64.b64encode(pubkey_pem).decode("ascii")
    finally:
        pub_path.unlink(missing_ok=True)


def normalize_hashes(values: list[str]) -> list[str]:
    normalized: list[str] = []
    for value in values:
        raw = str(value).strip()
        if raw.startswith("sha256:"):
            normalized.append(raw)
        else:
            normalized.append(f"sha256:{raw.lower()}")
    return normalized


def publish_rekor(
    *,
    rekor_url: str,
    entry_digest: str,
    signature_b64: str,
    signer_cert_path: Path,
    retries: int,
    backoff_s: float,
) -> dict[str, Any]:
    digest_hex = entry_digest.split(":", 1)[1]
    body = {
        "apiVersion": "0.0.1",
        "kind": "hashedrekord",
        "spec": {
            "data": {"hash": {"algorithm": "sha256", "value": digest_hex}},
            "signature": {
                "content": signature_b64,
                "publicKey": {"content": load_public_key_content(signer_cert_path)},
            },
        },
    }
    submit = retry_http_json(
        method="POST",
        url=f"{rekor_url.rstrip('/')}/api/v1/log/entries",
        payload=body,
        retries=retries,
        backoff_s=backoff_s,
    )

    if len(submit) != 1:
        raise RuntimeError(f"unexpected Rekor submit response shape: {submit}")
    entry_uuid = next(iter(submit.keys()))
    entry = submit[entry_uuid]
    verify = retry_http_json(
        method="GET",
        url=f"{rekor_url.rstrip('/')}/api/v1/log/entries/{entry_uuid}",
        payload=None,
        retries=retries,
        backoff_s=backoff_s,
    )
    record = verify.get(entry_uuid, entry)
    inclusion = (record.get("verification", {}) or {}).get("inclusionProof", {}) or {}
    consistency = (record.get("verification", {}) or {}).get("consistencyProof", {}) or {}
    log_index = int(record.get("logIndex", entry.get("logIndex", 0)))
    tree_size = int(inclusion.get("treeSize", log_index + 1))
    if tree_size <= log_index:
        tree_size = log_index + 1
    root_hash_hex = str(inclusion.get("rootHash", "")).strip().lower()
    if root_hash_hex and not root_hash_hex.startswith("sha256:"):
        root_hash = f"sha256:{root_hash_hex}"
    else:
        root_hash = root_hash_hex or "sha256:" + "0" * 64
    checkpoint_text = str(inclusion.get("checkpoint", "")).strip()
    if not checkpoint_text:
        checkpoint_text = f"rekor\n{tree_size}\n{root_hash}\n"
    integrated = record.get("integratedTime") or entry.get("integratedTime") or int(time.time())
    integrated_time = utc_from_epoch(int(integrated))
    inclusion_hashes = normalize_hashes(inclusion.get("hashes", []) or [])
    consistency_hashes = normalize_hashes(consistency.get("hashes", []) or [])

    proof = {
        "logId": "rekor",
        "logUrl": rekor_url.rstrip("/"),
        "entryUuid": entry_uuid,
        "entryDigest": entry_digest,
        "integratedTimeUtc": integrated_time,
        "logIndex": log_index,
        "treeSize": tree_size,
        "rootHash": root_hash,
        "leafHash": sha_prefixed_text(entry_digest),
        "checkpoint": checkpoint_text,
        "checkpointHash": sha_prefixed_text(checkpoint_text) if checkpoint_text else "",
        "inclusionPath": inclusion_hashes,
        "consistencyPath": consistency_hashes,
        "inclusionProofHash": json_hash({"hashes": inclusion_hashes, "rootHash": root_hash}),
        "consistencyProofHash": json_hash({"hashes": consistency_hashes, "rootHash": root_hash}),
        "rekorBodyHash": sha_prefixed_text(entry.get("body", "")),
    }
    return proof


def publish_mirror(
    *,
    mirror_url: str,
    entry_digest: str,
    retries: int,
    backoff_s: float,
) -> dict[str, Any]:
    submit = retry_http_json(
        method="POST",
        url=f"{mirror_url.rstrip('/')}/v1/entries",
        payload={"entryDigest": entry_digest},
        retries=retries,
        backoff_s=backoff_s,
    )
    entry_uuid = str(submit.get("entryUuid", "")).strip()
    if not entry_uuid:
        raise RuntimeError(f"mirror response missing entryUuid: {submit}")
    record = retry_http_json(
        method="GET",
        url=f"{mirror_url.rstrip('/')}/v1/entries/{entry_uuid}",
        payload=None,
        retries=retries,
        backoff_s=backoff_s,
    )
    return {
        "logId": "mirror",
        "logUrl": mirror_url.rstrip("/"),
        "entryUuid": entry_uuid,
        "entryDigest": entry_digest,
        "integratedTimeUtc": str(record.get("integratedTimeUtc", "")),
        "logIndex": int(record.get("logIndex", 0)),
        "treeSize": int(record.get("treeSize", 0)),
        "rootHash": str(record.get("rootHash", "")),
        "leafHash": str(record.get("leafHash", "")),
        "checkpoint": str(record.get("checkpoint", "")),
        "checkpointHash": str(record.get("checkpointHash", "")),
        "inclusionPath": normalize_hashes(record.get("inclusionPath", []) or []),
        "consistencyPath": normalize_hashes(record.get("consistencyPath", []) or []),
        "inclusionProofHash": str(record.get("inclusionProofHash", "")),
        "consistencyProofHash": str(record.get("consistencyProofHash", "")),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--entry-digest", required=True)
    parser.add_argument("--entry-signature-b64", required=True)
    parser.add_argument("--signer-cert", required=True)
    parser.add_argument("--rekor-url", default="https://rekor.sigstore.dev")
    parser.add_argument("--mirror-url", default="http://127.0.0.1:17777")
    parser.add_argument("--retries", type=int, default=3)
    parser.add_argument("--backoff-seconds", type=float, default=2.0)
    parser.add_argument("--output", required=True)
    args = parser.parse_args()

    signer_cert = Path(args.signer_cert).resolve()
    if not signer_cert.exists():
        raise SystemExit(f"signer cert missing: {signer_cert}")

    rekor = publish_rekor(
        rekor_url=args.rekor_url,
        entry_digest=args.entry_digest,
        signature_b64=args.entry_signature_b64,
        signer_cert_path=signer_cert,
        retries=args.retries,
        backoff_s=args.backoff_seconds,
    )
    mirror = publish_mirror(
        mirror_url=args.mirror_url,
        entry_digest=args.entry_digest,
        retries=args.retries,
        backoff_s=args.backoff_seconds,
    )

    payload = {"rekor": rekor, "mirror": mirror}
    Path(args.output).write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
