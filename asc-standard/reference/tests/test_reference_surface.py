#!/usr/bin/env python3
"""Integration tests for runnable reference surface outside kernel."""

from __future__ import annotations

import unittest

from reference.adapters.runtime import normalize_to_kernel_intent
from reference.interlock.runtime import InterlockGate
from reference.supervisor.runtime import EvidenceRefs, SupervisorRuntime


class ReferenceSurfaceTests(unittest.TestCase):
    def test_adapter_normalization_enforces_authority(self) -> None:
        intent = normalize_to_kernel_intent(
            {
                "profile": "uas-small",
                "missionId": "mission-001",
                "modeRequest": "AUTO",
                "commandVector": [0.5, 0.0, -0.2],
            }
        )
        self.assertEqual(intent["authority_source"], "safety_kernel")
        self.assertEqual(intent["profile"], "uas-small")
        self.assertTrue(intent["timestamp_utc"].endswith("Z"))

    def test_adapter_rejects_non_kernel_authority(self) -> None:
        with self.assertRaisesRegex(ValueError, "authority_source must be safety_kernel"):
            normalize_to_kernel_intent(
                {
                    "profile": "fixed-wing",
                    "missionId": "mission-002",
                    "modeRequest": "AUTO",
                    "commandVector": [1.0],
                    "authoritySource": "pilot_override",
                }
            )

    def test_interlock_latches_on_authority_violation(self) -> None:
        gate = InterlockGate(min_heartbeat_hz=10.0)
        first = gate.evaluate(
            authority_source="pilot_override",
            heartbeat_hz=12.0,
            within_safety_envelope=True,
            interlock_enabled=True,
        )
        self.assertFalse(first.allowed)
        self.assertTrue(first.latched)
        self.assertEqual(first.reason, "authority_exclusivity_violation")

        second = gate.evaluate(
            authority_source="safety_kernel",
            heartbeat_hz=12.0,
            within_safety_envelope=True,
            interlock_enabled=True,
        )
        self.assertFalse(second.allowed)
        self.assertEqual(second.reason, "latched_shutdown")

        gate.reset_latch()
        third = gate.evaluate(
            authority_source="safety_kernel",
            heartbeat_hz=12.0,
            within_safety_envelope=True,
            interlock_enabled=True,
        )
        self.assertTrue(third.allowed)
        self.assertEqual(third.reason, "authorized")

    def test_supervisor_incident_flow_includes_required_refs(self) -> None:
        supervisor = SupervisorRuntime()
        gate = InterlockGate(min_heartbeat_hz=10.0)
        decision = gate.evaluate(
            authority_source="safety_kernel",
            heartbeat_hz=12.5,
            within_safety_envelope=False,
            interlock_enabled=True,
        )
        refs = EvidenceRefs(
            evidence_map_hash="sha256:" + "1" * 64,
            signed_log_segment_hash="sha256:" + "2" * 64,
            replay_recipe_hash="sha256:" + "3" * 64,
            attestation_hash="sha256:" + "4" * 64,
        )
        initial = supervisor.build_incident_initial(
            system_id="sys:hybrid-vtol:alpha",
            profile="hybrid-vtol",
            severity="S2",
            narrative="Envelope breach recovered by interlock clamp.",
            refs=refs,
            decision=decision,
        )
        self.assertEqual(initial["templateId"], "tasc-incident-initial-v0.1")
        self.assertEqual(initial["reportingWindowsDays"], {"standard": 15, "widespread": 2, "fatal": 10})
        self.assertIn("evidenceRefs", initial)
        self.assertEqual(initial["evidenceRefs"]["evidenceMapHash"], refs.evidence_map_hash)

        pack = supervisor.build_incident_pack(
            initial=initial, corrective_action_plan_hash="sha256:" + "5" * 64
        )
        self.assertEqual(pack["templateId"], "tasc-incident-pack-v0.1")
        self.assertIn("CorrectiveActionPlan", pack["requiredArtifacts"])
        self.assertEqual(pack["fullPackSlaDays"], 10)


if __name__ == "__main__":
    unittest.main()
