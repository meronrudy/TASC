use std::path::PathBuf;

use tasc_sdk::{commands, SdkClient, SdkConfig, SupportBundleRequest};

#[test]
fn builds_support_bundle_args() {
    let request = SupportBundleRequest {
        target_path: Some(PathBuf::from("examples/minimal-local")),
        archive: true,
        output: Some(PathBuf::from("bundle.tar.gz")),
        redact: true,
        timeout_ms: None,
    };

    let args = commands::support_bundle::build_args(&request).expect("build args");
    assert_eq!(
        args,
        vec![
            "support-bundle",
            "examples/minimal-local",
            "--archive",
            "--output",
            "bundle.tar.gz",
            "--redact"
        ]
    );
}

#[test]
fn runs_support_bundle() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let request = SupportBundleRequest {
        target_path: Some(PathBuf::from("examples/minimal-local")),
        archive: false,
        output: None,
        redact: false,
        timeout_ms: None,
    };

    let response = client.support_bundle(&request).expect("support-bundle");
    let bundle_path = response.bundle_path.expect("bundle path");
    assert!(bundle_path.exists());
    let metadata = bundle_path.metadata().expect("metadata");
    assert!(metadata.len() > 0);
}
