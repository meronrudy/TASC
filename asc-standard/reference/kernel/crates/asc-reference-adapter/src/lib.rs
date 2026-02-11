use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterEnvelope {
    pub profile: String,
    #[serde(rename = "missionId")]
    pub mission_id: String,
    #[serde(rename = "modeRequest")]
    pub mode_request: String,
    #[serde(rename = "commandVector")]
    pub command_vector: Vec<f64>,
    #[serde(default = "default_authority", rename = "authoritySource")]
    pub authority_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelIntentEnvelope {
    #[serde(rename = "contractVersion")]
    pub contract_version: String,
    pub profile: String,
    #[serde(rename = "mission_id")]
    pub mission_id: String,
    #[serde(rename = "mode_request")]
    pub mode_request: String,
    #[serde(rename = "authority_source")]
    pub authority_source: String,
    #[serde(rename = "command_vector")]
    pub command_vector: Vec<f64>,
    #[serde(rename = "timestamp_utc")]
    pub timestamp_utc: String,
}

fn default_authority() -> String {
    "safety_kernel".to_string()
}

pub fn normalize_to_kernel_intent(
    payload: &AdapterEnvelope,
    timestamp_utc: String,
) -> Result<KernelIntentEnvelope> {
    if payload.authority_source != "safety_kernel" {
        bail!(
            "authority_source must be safety_kernel, got {}",
            payload.authority_source
        );
    }
    if payload.command_vector.is_empty() {
        bail!("command_vector cannot be empty");
    }
    Ok(KernelIntentEnvelope {
        contract_version: "1.0.0".to_string(),
        profile: payload.profile.clone(),
        mission_id: payload.mission_id.clone(),
        mode_request: payload.mode_request.clone(),
        authority_source: payload.authority_source.clone(),
        command_vector: payload.command_vector.clone(),
        timestamp_utc,
    })
}

#[cfg(test)]
mod tests {
    use super::{normalize_to_kernel_intent, AdapterEnvelope};

    #[test]
    fn normalize_enforces_authority() {
        let payload = AdapterEnvelope {
            profile: "uas-small".into(),
            mission_id: "mission-1".into(),
            mode_request: "AUTO".into(),
            command_vector: vec![0.1, 0.2, 0.3],
            authority_source: "pilot_override".into(),
        };
        let err = normalize_to_kernel_intent(&payload, "2026-02-11T00:00:00Z".into())
            .expect_err("must reject non-kernel authority");
        assert!(err
            .to_string()
            .contains("authority_source must be safety_kernel"));
    }
}
