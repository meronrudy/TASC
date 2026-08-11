use std::fs;

use serde_json::json;
use tasc_sdk::reports::{locate, read_last_explain, read_last_report, summarize};
use tempfile::TempDir;

#[test]
fn locate_handles_missing_state_files() {
    let dir = TempDir::new().expect("temp dir");
    let found = locate(dir.path());
    assert!(found.report_path.is_none());
    assert!(found.explain_path.is_none());
}

#[test]
fn read_last_artifacts_reads_json_payloads() {
    let dir = TempDir::new().expect("temp dir");
    let state = dir.path().join(".tasc");
    fs::create_dir_all(&state).expect("state dir");

    fs::write(
        state.join("last-report.json"),
        serde_json::to_vec(&json!({"result":"PASS","verdict":"PASS","failures":[]}))
            .expect("serialize report"),
    )
    .expect("write report");
    fs::write(
        state.join("last-explain.json"),
        serde_json::to_vec(&json!({"result":"PASS","summary":{"failedChecks":0}}))
            .expect("serialize explain"),
    )
    .expect("write explain");

    let report = read_last_report(dir.path()).expect("read last report");
    let explain = read_last_explain(dir.path()).expect("read last explain");

    assert_eq!(report["result"], "PASS");
    assert_eq!(explain["result"], "PASS");
}

#[test]
fn summarize_extracts_result_verdict_and_findings() {
    let report = json!({
        "result": "FAIL",
        "verdict": "FAIL",
        "findings": [{"code":"A"},{"code":"B"}],
    });
    let summary = summarize(&report);

    assert_eq!(summary.result, "FAIL");
    assert_eq!(summary.verdict, "FAIL");
    assert_eq!(summary.findings_count, 2);
}

#[test]
fn summarize_uses_failures_when_findings_missing() {
    let report = json!({
        "status": "WARN",
        "failures": [{"code":"X"}],
    });
    let summary = summarize(&report);

    assert_eq!(summary.result, "WARN");
    assert_eq!(summary.verdict, "WARN");
    assert_eq!(summary.findings_count, 1);
}
