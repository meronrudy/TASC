use std::fs;

use tasc_sdk::artifact_templates::{build_template, write_template, TemplateKind, TemplateRequest};
use tasc_sdk::{verify_path, ExplainRequest, OutputFormat, SdkClient, SdkConfig, VerifyOptions};
use tempfile::TempDir;

fn request(seed: u64, include_signature_stub: bool) -> TemplateRequest {
    TemplateRequest {
        kind: TemplateKind::EuReadinessCertificate,
        seed,
        trust_mode: Some("dev-local".to_string()),
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        include_signature_stub,
    }
}

#[test]
fn templates_are_deterministic_for_same_request() {
    let a = build_template(&request(7, true)).expect("template a");
    let b = build_template(&request(7, true)).expect("template b");
    assert_eq!(a.payload_bytes, b.payload_bytes);
}

#[test]
fn templates_change_with_seed() {
    let a = build_template(&request(7, false)).expect("template a");
    let b = build_template(&request(8, false)).expect("template b");
    assert_ne!(a.payload_bytes, b.payload_bytes);
}

#[test]
fn template_contains_expected_metadata() {
    let artifact = build_template(&request(42, true)).expect("template");
    assert_eq!(artifact.kind, TemplateKind::EuReadinessCertificate);
    assert_eq!(artifact.name, "EU Readiness Certificate");
    assert_eq!(artifact.payload_json["kind"], "eu-readiness-certificate");
    assert_eq!(artifact.payload_json["trustMode"], "dev-local");
    assert!(artifact.payload_json.get("signatureStub").is_some());
}

#[test]
fn write_template_persists_payload_bytes() {
    let artifact = build_template(&request(9, false)).expect("template");
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("template.json");
    write_template(&path, &artifact).expect("write template");
    let written = fs::read(&path).expect("read template file");

    let mut expected = artifact.payload_bytes.clone();
    expected.push(b'\n');
    assert_eq!(written, expected);
}

#[test]
fn template_runs_through_verify_and_explain_seams() {
    let artifact = build_template(&TemplateRequest {
        kind: TemplateKind::UnderwriterConfidencePacket,
        seed: 2026,
        trust_mode: Some("dev-local".to_string()),
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        include_signature_stub: true,
    })
    .expect("template artifact");

    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("template-underwriter.json");
    write_template(&path, &artifact).expect("write template");

    let verify = verify_path(&path, &VerifyOptions::default()).expect("verify via seam");
    assert!(!verify.status.is_empty());

    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let explain = client
        .explain(&ExplainRequest {
            report_path: verify.report_path,
            format: OutputFormat::Json,
            renderer: None,
            timeout_ms: None,
        })
        .expect("explain via seam");
    assert!(!explain.result.is_empty());
}
