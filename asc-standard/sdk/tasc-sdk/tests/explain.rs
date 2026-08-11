use std::path::PathBuf;

use tasc_sdk::{commands, ExplainRequest, OutputFormat, SdkClient, SdkConfig, VerifyRequest};

#[test]
fn builds_explain_args() {
    let request = ExplainRequest {
        report_path: Some(PathBuf::from("reports/report.json")),
        format: OutputFormat::Json,
        renderer: Some("default".to_string()),
        timeout_ms: None,
    };

    let args = commands::explain::build_args(&request).expect("build args");
    assert_eq!(
        args,
        vec![
            "explain",
            "reports/report.json",
            "--format",
            "json",
            "--renderer",
            "default"
        ]
    );
}

#[test]
fn runs_explain_after_verify() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let verify_request = VerifyRequest {
        target_path: PathBuf::from("examples/minimal-local"),
        profile_bundle: None,
        trust_mode: None,
        profile: None,
        timeout_ms: None,
        format: OutputFormat::Json,
    };
    let _ = client.verify(&verify_request).expect("verify");

    let explain_request = ExplainRequest {
        report_path: None,
        format: OutputFormat::Json,
        renderer: None,
        timeout_ms: None,
    };
    let response = client.explain(&explain_request).expect("explain");
    assert!(!response.result.is_empty());
    assert!(response.json.is_some());
}
