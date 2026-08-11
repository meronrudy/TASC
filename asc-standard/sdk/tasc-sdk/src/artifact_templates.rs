use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde_json::{json, Value};

use crate::SdkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateKind {
    EuReadinessCertificate,
    UnderwriterConfidencePacket,
    SimplexReversionPolicy,
    EvidenceMapHelloWorld,
}

impl TemplateKind {
    pub fn name(self) -> &'static str {
        match self {
            TemplateKind::EuReadinessCertificate => "EU Readiness Certificate",
            TemplateKind::UnderwriterConfidencePacket => "Underwriter Confidence Packet",
            TemplateKind::SimplexReversionPolicy => "Simplex Reversion Policy",
            TemplateKind::EvidenceMapHelloWorld => "Evidence Map Hello World",
        }
    }

    fn slug(self) -> &'static str {
        match self {
            TemplateKind::EuReadinessCertificate => "eu-readiness-certificate",
            TemplateKind::UnderwriterConfidencePacket => "underwriter-confidence-packet",
            TemplateKind::SimplexReversionPolicy => "simplex-reversion-policy",
            TemplateKind::EvidenceMapHelloWorld => "evidence-map-hello-world",
        }
    }

    fn tag(self) -> u64 {
        match self {
            TemplateKind::EuReadinessCertificate => 0xA1,
            TemplateKind::UnderwriterConfidencePacket => 0xB2,
            TemplateKind::SimplexReversionPolicy => 0xC3,
            TemplateKind::EvidenceMapHelloWorld => 0xD4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct TemplateRequest {
    pub kind: TemplateKind,
    pub seed: u64,
    pub trust_mode: Option<String>,
    pub profile_bundle: Option<String>,
    pub include_signature_stub: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TemplateArtifact {
    pub kind: TemplateKind,
    pub name: String,
    pub payload_json: Value,
    pub payload_bytes: Vec<u8>,
}

pub fn build_template(request: &TemplateRequest) -> Result<TemplateArtifact, SdkError> {
    let trust_mode = request
        .trust_mode
        .clone()
        .unwrap_or_else(|| "dev-local".to_string());
    let profile_bundle = request
        .profile_bundle
        .clone()
        .unwrap_or_else(|| "dev-local@1.0.0".to_string());

    let template_name = request.kind.name().to_string();
    let artifact_id = deterministic_hex(request.seed ^ request.kind.tag());

    let mut payload = json!({
        "templateVersion": "1.0.0",
        "kind": request.kind.slug(),
        "name": template_name,
        "seed": request.seed,
        "artifactId": artifact_id,
        "trustMode": trust_mode,
        "profileBundle": profile_bundle,
        "artifact": artifact_fields(request.kind, request.seed),
    });

    if request.include_signature_stub {
        payload["signatureStub"] = json!({
            "algorithm": "ES256",
            "kid": format!("demo-{}", deterministic_hex(request.seed ^ 0x5A5A)),
            "signature": "NON_PRODUCTION_SIGNATURE_STUB",
        });
    }

    let payload_bytes =
        serde_json::to_vec(&payload).map_err(|err| SdkError::ParseError(err.to_string()))?;

    Ok(TemplateArtifact {
        kind: request.kind,
        name: request.kind.name().to_string(),
        payload_json: payload,
        payload_bytes,
    })
}

pub fn write_template(path: &Path, artifact: &TemplateArtifact) -> Result<(), SdkError> {
    let mut file = File::create(path)?;
    file.write_all(&artifact.payload_bytes)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

fn artifact_fields(kind: TemplateKind, seed: u64) -> Value {
    match kind {
        TemplateKind::EuReadinessCertificate => json!({
            "certificate": format!("EU-READY-{}", deterministic_hex(seed ^ 0x11)),
            "jurisdiction": "EASA",
            "readinessScore": bounded_metric(seed, 60, 98),
        }),
        TemplateKind::UnderwriterConfidencePacket => json!({
            "packet": format!("UW-CONF-{}", deterministic_hex(seed ^ 0x22)),
            "riskBand": if bounded_metric(seed, 0, 100) > 70 { "LOW" } else { "MEDIUM" },
            "confidence": bounded_metric(seed ^ 0x33, 50, 99),
        }),
        TemplateKind::SimplexReversionPolicy => json!({
            "policy": format!("SIMPLEX-{}", deterministic_hex(seed ^ 0x44)),
            "trigger": "heartbeat.lost",
            "fallbackMode": "safe-hover",
        }),
        TemplateKind::EvidenceMapHelloWorld => json!({
            "mapId": format!("EVIDENCE-{}", deterministic_hex(seed ^ 0x55)),
            "state": "hello-world",
            "tuple": ["S", "F", "E", "G", "I"],
        }),
    }
}

fn deterministic_hex(seed: u64) -> String {
    let mut value = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value ^= value >> 30;
    value = value.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^= value >> 31;
    format!("{value:016x}")
}

fn bounded_metric(seed: u64, min: u32, max: u32) -> u32 {
    let span = max.saturating_sub(min).max(1);
    min + (seed as u32 % (span + 1))
}
