#!/usr/bin/env python3
"""Reference adapter runtime for ASC kernel-facing intents."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any


@dataclass(frozen=True)
class AdapterEnvelope:
    profile: str
    mission_id: str
    mode_request: str
    command_vector: list[float]
    authority_source: str = "safety_kernel"


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def normalize_to_kernel_intent(payload: dict[str, Any]) -> dict[str, Any]:
    """Normalize platform payload into a kernel intent contract."""
    envelope = AdapterEnvelope(
        profile=str(payload["profile"]),
        mission_id=str(payload["missionId"]),
        mode_request=str(payload["modeRequest"]),
        command_vector=[float(value) for value in payload["commandVector"]],
        authority_source=str(payload.get("authoritySource", "safety_kernel")),
    )
    if envelope.authority_source != "safety_kernel":
        raise ValueError(f"authority_source must be safety_kernel, got {envelope.authority_source}")
    if not envelope.command_vector:
        raise ValueError("command_vector cannot be empty")

    return {
        "contractVersion": "1.0.0",
        "profile": envelope.profile,
        "mission_id": envelope.mission_id,
        "mode_request": envelope.mode_request,
        "authority_source": envelope.authority_source,
        "command_vector": envelope.command_vector,
        "timestamp_utc": utc_now_iso(),
    }
