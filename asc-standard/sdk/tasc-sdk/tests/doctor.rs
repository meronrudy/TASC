use tasc_sdk::{commands, DoctorRequest, OutputFormat, SdkClient, SdkConfig};

#[test]
fn builds_doctor_args() {
    let request = DoctorRequest {
        operation: Some("verify".to_string()),
        profile_bundle: Some("bundle.json".to_string()),
        trust_mode: Some("test".to_string()),
        format: OutputFormat::Json,
        timeout_ms: None,
    };

    let args = commands::doctor::build_args(&request).expect("build args");
    assert_eq!(
        args,
        vec![
            "doctor",
            "--operation",
            "verify",
            "--profile-bundle",
            "bundle.json",
            "--trust-mode",
            "test",
            "--format",
            "json"
        ]
    );
}

#[test]
fn runs_doctor_json() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let request = DoctorRequest {
        operation: None,
        profile_bundle: None,
        trust_mode: None,
        format: OutputFormat::Json,
        timeout_ms: None,
    };

    let response = client.doctor(&request).expect("doctor");
    assert!(!response.result.is_empty());
    assert!(response.json.is_some());
}
