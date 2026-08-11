use std::fs;

use serde_json::json;
use tempfile::TempDir;

use tasc_sdk::export::{export_html, export_json};

#[test]
fn exports_json_with_fsync() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("output.json");
    let payload = json!({"ok": true, "value": 42});

    export_json(&path, &payload).expect("export json");
    let contents = fs::read_to_string(path).expect("read json");
    assert!(contents.contains("\"ok\""));
    assert!(contents.contains("42"));
}

#[test]
fn exports_html_with_template() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("report.html");

    export_html(&path, "Test Report", "Hello").expect("export html");
    let contents = fs::read_to_string(path).expect("read html");
    assert!(contents.contains("<title>Test Report</title>"));
    assert!(contents.contains("Hello"));
}
