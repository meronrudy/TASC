use std::fs;

use tasc_sdk::test_helpers::{assert_verify_and_explain, seeded_bundle_bytes, seeded_bundle_file};
use tasc_sdk::{SdkClient, SdkConfig};

#[test]
fn seeded_bundle_is_deterministic() {
    assert_eq!(seeded_bundle_bytes(), seeded_bundle_bytes());
}

#[test]
fn seeded_bundle_file_writes_contents() {
    let file = seeded_bundle_file().expect("bundle file");
    let contents = fs::read_to_string(file.path()).expect("read bundle");
    assert!(contents.contains("\"seeded\""));
}

#[test]
fn ci_helper_runs_verify_and_explain() {
    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let path = std::path::Path::new("examples/minimal-local");
    let (verify, explain) = assert_verify_and_explain(&client, path).expect("verify+explain");
    assert!(!verify.status.is_empty());
    assert!(!explain.result.is_empty());
}
