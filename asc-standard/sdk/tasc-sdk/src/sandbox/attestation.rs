use crate::SdkError;

pub const NON_PRODUCTION_MARKER: &str = "NON_PRODUCTION";

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SandboxAttestation {
    pub marker: &'static str,
    pub trust_mode: Option<String>,
    pub evidence: String,
}

pub fn ensure_non_production(trust_mode: Option<&str>) -> Result<(), SdkError> {
    if matches!(trust_mode, Some("prod-hsm")) {
        return Err(SdkError::InvalidConfig(
            "sandbox attestation is not allowed with trust_mode=prod-hsm".to_string(),
        ));
    }
    Ok(())
}

pub fn attest(trust_mode: Option<&str>) -> Result<SandboxAttestation, SdkError> {
    ensure_non_production(trust_mode)?;
    Ok(SandboxAttestation {
        marker: NON_PRODUCTION_MARKER,
        trust_mode: trust_mode.map(|mode| mode.to_string()),
        evidence: "sandbox-attestation".to_string(),
    })
}
