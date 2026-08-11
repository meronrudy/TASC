use std::path::PathBuf;

use tasc_sdk::{commands, CiPreflightRequest, OutputFormat, SdkClient, SdkConfig};

#[test]
fn builds_ci_preflight_args() {
    let request = CiPreflightRequest {
        examples: Some(vec![
            PathBuf::from("examples/minimal-local"),
            PathBuf::from("examples/minimal-signed"),
        ]),
        format: OutputFormat::Json,
        strict: true,
        output: Some(PathBuf::from("preflight.json")),
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        trust_mode: Some("dev-local".to_string()),
        timeout_ms: None,
    };

    let args = commands::ci_preflight::build_args(&request).expect("build args");
    assert_eq!(
        args,
        vec![
            "ci-preflight",
            "--examples",
            "examples/minimal-local",
            "examples/minimal-signed",
            "--format",
            "json",
            "--strict",
            "--output",
            "preflight.json",
            "--profile-bundle",
            "dev-local@1.0.0",
            "--trust-mode",
            "dev-local"
        ]
    );
}

#[test]
fn ci_preflight_strict_failure_sets_exit_code() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let request = CiPreflightRequest {
        examples: Some(vec![PathBuf::from("examples/does-not-exist")]),
        format: OutputFormat::Json,
        strict: true,
        output: None,
        profile_bundle: None,
        trust_mode: None,
        timeout_ms: None,
    };

    let response = client.ci_preflight(&request).expect("ci-preflight");
    let exit_code = response.output.exit_code.unwrap_or(0);
    assert_ne!(exit_code, 0);
    assert!(!response.output.stderr.trim().is_empty());
}
