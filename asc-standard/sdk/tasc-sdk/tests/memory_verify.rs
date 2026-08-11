use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tasc_sdk::memory_verify::{verify_bytes_with, verify_string_with};
use tasc_sdk::{SdkError, VerifyOptions, VerifyResponse};

#[test]
fn verify_string_cleans_temp_file() {
    let observed: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    let observed_runner = observed.clone();

    let response = verify_string_with("hello", &VerifyOptions::default(), |path, _| {
        assert!(path.exists());
        *observed_runner.lock().unwrap() = Some(path.to_path_buf());
        Ok(VerifyResponse {
            status: "PASS".to_string(),
            verdict: "OK".to_string(),
            report_path: None,
            output: None,
            json: None,
        })
    })
    .expect("verify_string should succeed");

    assert_eq!(response.status, "PASS");
    let path = observed.lock().unwrap().clone().expect("path captured");
    assert!(!path.exists());
}

#[test]
fn verify_bytes_cleans_temp_file_on_error() {
    let observed: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    let observed_runner = observed.clone();

    let result = verify_bytes_with(&[1, 2, 3], &VerifyOptions::default(), |path, _| {
        assert!(path.exists());
        *observed_runner.lock().unwrap() = Some(path.to_path_buf());
        Err(SdkError::NotImplemented("forced"))
    });

    assert!(result.is_err());
    let path = observed.lock().unwrap().clone().expect("path captured");
    assert!(!path.exists());
}
