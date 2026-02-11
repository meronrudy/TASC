#!/usr/bin/env python3
"""Native PKCS#11 helper for assurancepack signing and certificate retrieval."""

from __future__ import annotations

import os
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path

import yaml


@dataclass(frozen=True)
class Pkcs11Profile:
    module: str
    slot: str | None
    token_label: str | None
    key_label: str
    cert_label: str
    pin_env: str
    mechanism: str
    intermediate_chain_path: str | None
    root_bundle_path: str | None


def load_profile(path: Path) -> Pkcs11Profile:
    payload = yaml.safe_load(path.read_text(encoding="utf-8")) or {}
    required = ["module", "key_label", "cert_label", "pin_env", "mechanism"]
    missing = [field for field in required if not str(payload.get(field, "")).strip()]
    if missing:
        raise RuntimeError(f"pkcs11 profile missing fields: {', '.join(missing)}")

    slot = payload.get("slot")
    token_label = payload.get("token_label")
    if slot is not None:
        slot = str(slot)
    if token_label is not None:
        token_label = str(token_label)

    return Pkcs11Profile(
        module=str(payload["module"]).strip(),
        slot=slot,
        token_label=token_label,
        key_label=str(payload["key_label"]).strip(),
        cert_label=str(payload["cert_label"]).strip(),
        pin_env=str(payload["pin_env"]).strip(),
        mechanism=str(payload["mechanism"]).strip(),
        intermediate_chain_path=_none_if_blank(payload.get("intermediate_chain_path")),
        root_bundle_path=_none_if_blank(payload.get("root_bundle_path")),
    )


def _none_if_blank(value: object) -> str | None:
    text = str(value).strip() if value is not None else ""
    return text or None


class Pkcs11Client:
    def __init__(
        self,
        *,
        module: str,
        slot: str | None,
        token_label: str | None,
        key_label: str,
        cert_label: str,
        pin_env: str,
        mechanism: str,
    ) -> None:
        self.module = module
        self.slot = slot
        self.token_label = token_label
        self.key_label = key_label
        self.cert_label = cert_label
        self.pin_env = pin_env
        self.mechanism = mechanism

        if not self.slot and not self.token_label:
            raise RuntimeError("pkcs11 client requires slot or token_label")
        if self.pin_env not in os.environ:
            raise RuntimeError(f"pkcs11 pin env var missing: {self.pin_env}")

    @property
    def pin(self) -> str:
        pin = os.environ.get(self.pin_env, "")
        if not pin:
            raise RuntimeError(f"pkcs11 pin env var is empty: {self.pin_env}")
        return pin

    def sign(self, payload: bytes) -> bytes:
        with tempfile.NamedTemporaryFile(delete=False) as in_file, tempfile.NamedTemporaryFile(
            delete=False
        ) as out_file:
            in_path = Path(in_file.name)
            out_path = Path(out_file.name)
            in_file.write(payload)
            in_file.flush()

        try:
            command = self._base_command() + [
                "--sign",
                "--label",
                self.key_label,
                "--mechanism",
                self.mechanism,
                "--input-file",
                str(in_path),
                "--output-file",
                str(out_path),
            ]
            self._run(command, "pkcs11 sign")
            return out_path.read_bytes()
        finally:
            in_path.unlink(missing_ok=True)
            out_path.unlink(missing_ok=True)

    def export_certificate_pem(self, cert_label: str, output_path: Path) -> Path:
        with tempfile.NamedTemporaryFile(delete=False) as cert_file:
            cert_path = Path(cert_file.name)
        try:
            command = self._base_command() + [
                "--read-object",
                "--type",
                "cert",
                "--label",
                cert_label,
                "--output-file",
                str(cert_path),
            ]
            self._run(command, f"pkcs11 read cert {cert_label}")
            cert_bytes = cert_path.read_bytes()
            pem = _to_pem_certificate(cert_bytes)
            output_path.write_text(pem, encoding="utf-8")
            return output_path
        finally:
            cert_path.unlink(missing_ok=True)

    def _base_command(self) -> list[str]:
        command = [
            "pkcs11-tool",
            "--module",
            self.module,
            "--login",
            "--pin",
            self.pin,
        ]
        if self.slot:
            command.extend(["--slot", self.slot])
        if self.token_label:
            command.extend(["--token-label", self.token_label])
        return command

    @staticmethod
    def _run(command: list[str], label: str) -> None:
        proc = subprocess.run(command, capture_output=True, text=True, check=False)
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip()
            raise RuntimeError(f"{label} failed: {detail}")


def _to_pem_certificate(raw: bytes) -> str:
    if b"-----BEGIN CERTIFICATE-----" in raw:
        return raw.decode("utf-8")

    with tempfile.NamedTemporaryFile(delete=False) as der_file, tempfile.NamedTemporaryFile(
        delete=False
    ) as pem_file:
        der_path = Path(der_file.name)
        pem_path = Path(pem_file.name)
        der_file.write(raw)
        der_file.flush()

    try:
        proc = subprocess.run(
            [
                "openssl",
                "x509",
                "-inform",
                "DER",
                "-in",
                str(der_path),
                "-out",
                str(pem_path),
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if proc.returncode != 0:
            detail = (proc.stderr or proc.stdout or "").strip()
            raise RuntimeError(f"openssl x509 DER->PEM conversion failed: {detail}")
        return pem_path.read_text(encoding="utf-8")
    finally:
        der_path.unlink(missing_ok=True)
        pem_path.unlink(missing_ok=True)


def compose_chain_file(
    *,
    signer_cert_pem_path: Path,
    chain_output_path: Path,
    intermediate_chain_path: Path | None,
) -> Path:
    chunks = [signer_cert_pem_path.read_text(encoding="utf-8").strip()]
    if intermediate_chain_path and intermediate_chain_path.exists():
        chunks.append(intermediate_chain_path.read_text(encoding="utf-8").strip())
    chain_output_path.write_text("\n".join(chunks).strip() + "\n", encoding="utf-8")
    return chain_output_path
