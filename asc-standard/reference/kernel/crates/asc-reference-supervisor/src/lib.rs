use asc_reference_interlock::InterlockDecision;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRefs {
    pub evidence_map_hash: String,
    pub signed_log_segment_hash: String,
    pub replay_recipe_hash: String,
    pub attestation_hash: String,
}

pub fn build_incident_initial(
    system_id: &str,
    profile: &str,
    severity: &str,
    narrative: &str,
    refs: &EvidenceRefs,
    decision: &InterlockDecision,
    timestamp_utc: &str,
) -> serde_json::Value {
    json!({
        "templateId": "tasc-incident-initial-v0.1",
        "systemId": system_id,
        "profile": profile,
        "timestampUtc": timestamp_utc,
        "severity": severity,
        "narrative": narrative,
        "interlockDecision": {
            "allowed": decision.allowed,
            "reason": decision.reason,
            "latched": decision.latched,
        },
        "evidenceRefs": {
            "evidenceMapHash": refs.evidence_map_hash,
            "signedLogSegmentHash": refs.signed_log_segment_hash,
            "replayRecipeHash": refs.replay_recipe_hash,
            "attestationHash": refs.attestation_hash,
        },
        "reportingWindowsDays": {"standard": 15, "widespread": 2, "fatal": 10},
    })
}

pub fn build_incident_pack(
    initial: &serde_json::Value,
    corrective_action_plan_hash: &str,
    timestamp_utc: &str,
) -> serde_json::Value {
    let evidence_map = initial
        .pointer("/evidenceRefs/evidenceMapHash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let signed_log = initial
        .pointer("/evidenceRefs/signedLogSegmentHash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let replay = initial
        .pointer("/evidenceRefs/replayRecipeHash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    let attestation = initial
        .pointer("/evidenceRefs/attestationHash")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    json!({
        "templateId": "tasc-incident-pack-v0.1",
        "initialRef": evidence_map,
        "timestampUtc": timestamp_utc,
        "requiredArtifacts": [
            "EvidenceMap",
            "SignedOperationalLog",
            "TransparencyProofs",
            "ReplayBundle",
            "ConfigurationBaseline",
            "CorrectiveActionPlan",
        ],
        "artifactRefs": {
            "EvidenceMap": evidence_map,
            "SignedOperationalLog": signed_log,
            "ReplayBundle": replay,
            "Attestation": attestation,
            "CorrectiveActionPlan": corrective_action_plan_hash,
        },
        "fullPackSlaDays": 10,
    })
}
