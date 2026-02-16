#!/usr/bin/env python3
"""File-backed append-only mirror transparency log service."""

from __future__ import annotations

import argparse
import hashlib
import json
import uuid
from datetime import datetime, timezone
from http import HTTPStatus
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from threading import RLock
from typing import Any
from urllib.parse import urlparse


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def sha256_hex(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha_prefixed(value: bytes) -> str:
    return f"sha256:{sha256_hex(value)}"


def leaf_hash(entry_digest: str) -> str:
    return sha_prefixed(f"leaf:{entry_digest}".encode("utf-8"))


def node_hash(left: str, right: str) -> str:
    material = f"node:{left}:{right}".encode("utf-8")
    return sha_prefixed(material)


def merkle_root(leaves: list[str]) -> str:
    if not leaves:
        return sha_prefixed(b"empty-tree")
    layer = list(leaves)
    while len(layer) > 1:
        next_layer: list[str] = []
        idx = 0
        while idx < len(layer):
            left = layer[idx]
            right = layer[idx + 1] if idx + 1 < len(layer) else left
            next_layer.append(node_hash(left, right))
            idx += 2
        layer = next_layer
    return layer[0]


def inclusion_path(leaves: list[str], index: int) -> list[str]:
    if index < 0 or index >= len(leaves):
        return []
    path: list[str] = []
    layer = list(leaves)
    pos = index
    while len(layer) > 1:
        sibling = pos ^ 1
        if sibling < len(layer):
            path.append(layer[sibling])
        else:
            path.append(layer[pos])
        next_layer: list[str] = []
        idx = 0
        while idx < len(layer):
            left = layer[idx]
            right = layer[idx + 1] if idx + 1 < len(layer) else left
            next_layer.append(node_hash(left, right))
            idx += 2
        layer = next_layer
        pos //= 2
    return path


class MirrorStore:
    def __init__(self, storage_dir: Path, log_id: str) -> None:
        self.storage_dir = storage_dir
        self.log_id = log_id
        self.entries_file = storage_dir / "entries.jsonl"
        # proof_for() calls checkpoint(), so we need a re-entrant lock to avoid self-deadlock.
        self.lock = RLock()
        storage_dir.mkdir(parents=True, exist_ok=True)
        self.entries: list[dict[str, Any]] = self._load_entries()

    def _load_entries(self) -> list[dict[str, Any]]:
        if not self.entries_file.exists():
            return []
        loaded: list[dict[str, Any]] = []
        for line in self.entries_file.read_text(encoding="utf-8").splitlines():
            if not line.strip():
                continue
            loaded.append(json.loads(line))
        return loaded

    def append(self, entry_digest: str) -> dict[str, Any]:
        with self.lock:
            entry_uuid = str(uuid.uuid4())
            record = {
                "uuid": entry_uuid,
                "entryDigest": entry_digest,
                "integratedTimeUtc": utc_now_iso(),
                "logIndex": len(self.entries),
            }
            self.entries.append(record)
            with self.entries_file.open("a", encoding="utf-8") as handle:
                handle.write(json.dumps(record, sort_keys=True) + "\n")
            return self.proof_for(record)

    def checkpoint(self) -> dict[str, Any]:
        with self.lock:
            leaf_nodes = [leaf_hash(entry["entryDigest"]) for entry in self.entries]
            tree_size = len(leaf_nodes)
            root = merkle_root(leaf_nodes)
            checkpoint = f"{self.log_id}\n{tree_size}\n{root}\n"
            return {
                "logId": self.log_id,
                "treeSize": tree_size,
                "rootHash": root,
                "checkpoint": checkpoint,
                "checkpointHash": sha_prefixed(checkpoint.encode("utf-8")),
                "generatedAtUtc": utc_now_iso(),
            }

    def proof_for(self, record: dict[str, Any]) -> dict[str, Any]:
        leaf_nodes = [leaf_hash(entry["entryDigest"]) for entry in self.entries]
        checkpoint = self.checkpoint()
        idx = int(record["logIndex"])
        inclusion = inclusion_path(leaf_nodes, idx)
        consistency: list[str] = []
        inclusion_material = json.dumps(
            {"hashes": inclusion, "rootHash": checkpoint["rootHash"]},
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        consistency_material = json.dumps(
            {"hashes": consistency, "rootHash": checkpoint["rootHash"]},
            sort_keys=True,
            separators=(",", ":"),
        ).encode("utf-8")
        return {
            "logId": self.log_id,
            "logUrl": "mirror://internal-file-backed",
            "entryUuid": record["uuid"],
            "entryDigest": record["entryDigest"],
            "integratedTimeUtc": record["integratedTimeUtc"],
            "logIndex": idx,
            "treeSize": checkpoint["treeSize"],
            "rootHash": checkpoint["rootHash"],
            "leafHash": leaf_nodes[idx],
            "inclusionPath": inclusion,
            "consistencyPath": consistency,
            "checkpoint": checkpoint["checkpoint"],
            "checkpointHash": checkpoint["checkpointHash"],
            "inclusionProofHash": sha_prefixed(inclusion_material),
            "consistencyProofHash": sha_prefixed(consistency_material),
        }

    def get_by_uuid(self, entry_uuid: str) -> dict[str, Any] | None:
        with self.lock:
            for entry in self.entries:
                if entry["uuid"] == entry_uuid:
                    return self.proof_for(entry)
        return None


def build_handler(store: MirrorStore):
    class Handler(BaseHTTPRequestHandler):
        server_version = "tasc-mirror/0.1"

        def _json(self, status: int, payload: dict[str, Any]) -> None:
            body = json.dumps(payload, sort_keys=True).encode("utf-8")
            self.send_response(status)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_POST(self) -> None:  # noqa: N802
            if self.path != "/v1/entries":
                self._json(HTTPStatus.NOT_FOUND, {"error": "not found"})
                return
            length = int(self.headers.get("Content-Length", "0"))
            raw = self.rfile.read(length)
            payload = json.loads(raw.decode("utf-8"))
            entry_digest = str(payload.get("entryDigest", "")).strip()
            if not entry_digest.startswith("sha256:"):
                self._json(HTTPStatus.BAD_REQUEST, {"error": "entryDigest must be sha256:*"})
                return
            proof = store.append(entry_digest)
            self._json(HTTPStatus.CREATED, proof)

        def do_GET(self) -> None:  # noqa: N802
            parsed = urlparse(self.path)
            if parsed.path == "/v1/checkpoint":
                self._json(HTTPStatus.OK, store.checkpoint())
                return
            if parsed.path.startswith("/v1/entries/"):
                entry_uuid = parsed.path.rsplit("/", 1)[-1]
                proof = store.get_by_uuid(entry_uuid)
                if not proof:
                    self._json(HTTPStatus.NOT_FOUND, {"error": "entry not found"})
                    return
                self._json(HTTPStatus.OK, proof)
                return
            self._json(HTTPStatus.NOT_FOUND, {"error": "not found"})

        def log_message(self, fmt: str, *args: object) -> None:
            return

    return Handler


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--storage-dir", default="conformance/mirror-log")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=17777)
    parser.add_argument("--log-id", default="mirror")
    args = parser.parse_args()

    store = MirrorStore(Path(args.storage_dir).resolve(), args.log_id)
    server = ThreadingHTTPServer((args.host, args.port), build_handler(store))
    print(f"mirror service listening on http://{args.host}:{args.port}", flush=True)
    server.serve_forever()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
