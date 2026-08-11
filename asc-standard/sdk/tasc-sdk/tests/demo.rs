use tasc_sdk::{commands, DemoRequest, OutputFormat, SdkClient, SdkConfig};

#[test]
fn builds_demo_requests() {
    let request = DemoRequest {
        format: OutputFormat::Json,
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        trust_mode: Some("dev-local".to_string()),
        timeout_ms: Some(10),
    };
    let (verify, explain) = commands::demo::build_demo_requests(&request);

    assert_eq!(
        verify.target_path.to_string_lossy(),
        "examples/minimal-local"
    );
    assert_eq!(verify.format, OutputFormat::Json);
    assert_eq!(verify.profile_bundle.as_deref(), Some("dev-local@1.0.0"));
    assert_eq!(verify.trust_mode.as_deref(), Some("dev-local"));
    assert_eq!(verify.timeout_ms, Some(10));
    assert_eq!(explain.format, OutputFormat::Json);
    assert_eq!(explain.timeout_ms, Some(10));
}

#[test]
fn runs_demo_flow() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let request = DemoRequest {
        format: OutputFormat::Json,
        profile_bundle: None,
        trust_mode: None,
        timeout_ms: None,
    };
    let response = client.demo(&request).expect("demo");
    assert!(!response.verify.verdict.is_empty());
    assert!(!response.explain.result.is_empty());
}
