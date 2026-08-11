use std::path::PathBuf;

use tasc_sdk::diagnostics::{is_healthy, run, DiagnosticSeverity};
use tasc_sdk::{SdkClient, SdkConfig};
use tempfile::TempDir;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("canonical repo root")
}

#[test]
fn diagnostics_pass_on_repo_root() {
    let root = repo_root();
    let report = run(&SdkConfig::default(), Some(&root)).expect("diagnostics report");
    assert!(is_healthy(&report));
    assert_eq!(report.result, "PASS");
}

#[test]
fn diagnostics_flags_missing_repo_root() {
    let temp = TempDir::new().expect("temp dir");
    let report = run(&SdkConfig::default(), Some(temp.path())).expect("diagnostics report");

    assert_eq!(report.result, "FAIL");
    assert!(report.findings.iter().any(|finding| {
        finding.code == "REPO_ROOT_NOT_FOUND" && finding.severity == DiagnosticSeverity::Error
    }));
}

#[test]
fn diagnostics_flags_missing_tasc_override() {
    let root = repo_root();
    let config = SdkConfig {
        repo_root: Some(root.clone()),
        tasc_path: Some(root.join("missing-tasc")),
        timeout_ms: None,
    };

    let report = run(&config, Some(&root)).expect("diagnostics report");
    assert_eq!(report.result, "FAIL");
    assert!(report
        .findings
        .iter()
        .any(|finding| finding.code == "TASC_NOT_FOUND"));
}

#[test]
fn client_exposes_diagnostics_wrapper() {
    let root = repo_root();
    let client = SdkClient::new(SdkConfig {
        repo_root: Some(root),
        tasc_path: None,
        timeout_ms: None,
    })
    .expect("client");

    let report = client.diagnostics().expect("client diagnostics");
    assert!(!report.result.is_empty());
}
