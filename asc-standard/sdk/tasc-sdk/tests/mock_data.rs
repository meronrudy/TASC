use std::fs;

use tasc_sdk::mock_data::{
    generate_bundle, to_verify_bytes, write_bundle, MockDataRequest, MockScenario,
};
use tasc_sdk::{verify_path, ExplainRequest, OutputFormat, SdkClient, SdkConfig, VerifyOptions};
use tempfile::TempDir;

#[test]
fn mock_data_is_deterministic_for_same_request() {
    let request = MockDataRequest {
        scenario: MockScenario::FlightLog,
        seed: 77,
        records: 6,
    };

    let a = generate_bundle(&request).expect("bundle a");
    let b = generate_bundle(&request).expect("bundle b");

    assert_eq!(a.bundle_bytes, b.bundle_bytes);
    assert_eq!(a.events_json, b.events_json);
}

#[test]
fn mock_data_shapes_match_scenario() {
    let flight = generate_bundle(&MockDataRequest {
        scenario: MockScenario::FlightLog,
        seed: 1,
        records: 1,
    })
    .expect("flight bundle");
    assert!(flight.events_json[0].get("altitudeM").is_some());

    let maintenance = generate_bundle(&MockDataRequest {
        scenario: MockScenario::MaintenanceLog,
        seed: 1,
        records: 1,
    })
    .expect("maintenance bundle");
    assert!(maintenance.events_json[0].get("component").is_some());

    let incident = generate_bundle(&MockDataRequest {
        scenario: MockScenario::IncidentReplay,
        seed: 1,
        records: 1,
    })
    .expect("incident bundle");
    assert!(incident.events_json[0].get("impactG").is_some());
}

#[test]
fn records_must_be_positive() {
    let err = generate_bundle(&MockDataRequest {
        scenario: MockScenario::FlightLog,
        seed: 2,
        records: 0,
    })
    .expect_err("records=0 should fail");

    match err {
        tasc_sdk::SdkError::InvalidConfig(message) => {
            assert!(message.contains("at least one record"))
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn write_and_verify_bytes_helpers_work() {
    let bundle = generate_bundle(&MockDataRequest {
        scenario: MockScenario::IncidentReplay,
        seed: 44,
        records: 3,
    })
    .expect("bundle");

    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("bundle.json");
    write_bundle(&path, &bundle).expect("write bundle");

    let written = fs::read(&path).expect("read bundle");
    let mut expected = bundle.bundle_bytes.clone();
    expected.push(b'\n');
    assert_eq!(written, expected);
    assert_eq!(to_verify_bytes(&bundle), bundle.bundle_bytes);
}

#[test]
fn generated_bundle_runs_through_verify_and_explain_seams() {
    let bundle = generate_bundle(&MockDataRequest {
        scenario: MockScenario::MaintenanceLog,
        seed: 404,
        records: 4,
    })
    .expect("bundle");

    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("generated-report.json");
    write_bundle(&path, &bundle).expect("write bundle");

    let verify = verify_path(&path, &VerifyOptions::default()).expect("verify via seam");
    assert!(!verify.status.is_empty());

    let client = SdkClient::new(SdkConfig::default()).expect("client");
    let explain = client
        .explain(&ExplainRequest {
            report_path: verify.report_path,
            format: OutputFormat::Json,
            renderer: None,
            timeout_ms: None,
        })
        .expect("explain via seam");
    assert!(!explain.result.is_empty());
}
