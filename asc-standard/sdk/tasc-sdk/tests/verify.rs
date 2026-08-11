use std::path::PathBuf;

use tasc_sdk::{commands, OutputFormat, SdkClient, SdkConfig, VerifyRequest};

#[test]
fn builds_verify_args() {
    let request = VerifyRequest {
        target_path: PathBuf::from("examples/minimal-local"),
        profile_bundle: Some("bundle.json".to_string()),
        trust_mode: Some("dev-local".to_string()),
        profile: None,
        timeout_ms: None,
        format: OutputFormat::Json,
    };

    let args = commands::verify::build_args(&request).expect("build args");
    assert_eq!(
        args,
        vec![
            "verify",
            "examples/minimal-local",
            "--profile-bundle",
            "bundle.json",
            "--trust-mode",
            "dev-local",
            "--format",
            "json"
        ]
    );
}

#[test]
fn runs_verify_minimal_local() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let request = VerifyRequest {
        target_path: PathBuf::from("examples/minimal-local"),
        profile_bundle: None,
        trust_mode: None,
        profile: None,
        timeout_ms: None,
        format: OutputFormat::Json,
    };

    let response = client.verify(&request).expect("verify");
    assert!(!response.verdict.is_empty());
    let report_exists = response
        .report_path
        .as_ref()
        .map(|path| path.exists())
        .unwrap_or(false);
    assert!(report_exists);
}
