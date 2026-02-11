#!/usr/bin/env python3
"""Reference supervisor orchestration for incident pack flow."""

from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime, timezone
from typing import Any

from reference.interlock.runtime import InterlockDecision


def utc_now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat().replace("+00:00", "Z")


@dataclass(frozen=True)
class EvidenceRefs:
    evidence_map_hash: str
    signed_log_segment_hash: str
    replay_recipe_hash: str
    attestation_hash: str


class SupervisorRuntime:
    def build_incident_initial(
        self,
        *,
        system_id: str,
        profile: str,
        severity: str,
        narrative: str,
        refs: EvidenceRefs,
        decision: InterlockDecision,
    ) -> dict[str, Any]:
        return {
            "templateId": "tasc-incident-initial-v0.1",
            "systemId": system_id,
            "profile": profile,
            "timestampUtc": utc_now_iso(),
            "severity": severity,
            "narrative": narrative,
            "interlockDecision": decision.as_dict(),
            "evidenceRefs": {
                "evidenceMapHash": refs.evidence_map_hash,
                "signedLogSegmentHash": refs.signed_log_segment_hash,
                "replayRecipeHash": refs.replay_recipe_hash,
                "attestationHash": refs.attestation_hash,
            },
            "reportingWindowsDays": {"standard": 15, "widespread": 2, "fatal": 10},
        }

    def build_incident_pack(
        self,
        *,
        initial: dict[str, Any],
        corrective_action_plan_hash: str,
    ) -> dict[str, Any]:
        return {
            "templateId": "tasc-incident-pack-v0.1",
            "initialRef": initial["evidenceRefs"]["evidenceMapHash"],
            "timestampUtc": utc_now_iso(),
            "requiredArtifacts": [
                "EvidenceMap",
                "SignedOperationalLog",
                "TransparencyProofs",
                "ReplayBundle",
                "ConfigurationBaseline",
                "CorrectiveActionPlan",
            ],
            "artifactRefs": {
                "EvidenceMap": initial["evidenceRefs"]["evidenceMapHash"],
                "SignedOperationalLog": initial["evidenceRefs"]["signedLogSegmentHash"],
                "ReplayBundle": initial["evidenceRefs"]["replayRecipeHash"],
                "Attestation": initial["evidenceRefs"]["attestationHash"],
                "CorrectiveActionPlan": corrective_action_plan_hash,
            },
            "fullPackSlaDays": 10,
        }
