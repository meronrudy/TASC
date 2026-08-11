use std::path::PathBuf;
use std::process::Command;

use tasc_sdk::{command_runner, exit_code_for, OutputFormat, SdkError};

#[test]
fn maps_command_failure_to_sdk_error() {
    let mut cmd = Command::new("ls");
    cmd.arg("does-not-exist");

    let err = command_runner::run(&mut cmd, None).expect_err("expected failure");
    match err {
        SdkError::CommandFailed { stdout, stderr, .. } => {
            assert!(stdout.trim().is_empty());
            assert!(stderr.contains("No such"));
        }
        other => panic!("unexpected error: {other:?}"),
    }
}

#[test]
fn maps_exit_codes() {
    assert_eq!(exit_code_for(&SdkError::InvalidConfig("bad".into())), 2);
    assert_eq!(
        exit_code_for(&SdkError::RepoNotFound(PathBuf::from("/"))),
        2
    );
    assert_eq!(
        exit_code_for(&SdkError::TascNotFound(PathBuf::from("/tasc"))),
        2
    );
    assert_eq!(
        exit_code_for(&SdkError::CommandFailed {
            code: Some(2),
            stdout: String::new(),
            stderr: String::new()
        }),
        3
    );
    assert_eq!(
        exit_code_for(&SdkError::Timeout {
            after_ms: 1,
            stdout: String::new(),
            stderr: String::new()
        }),
        3
    );
    assert_eq!(exit_code_for(&SdkError::ParseError("bad".into())), 3);
    assert_eq!(exit_code_for(&SdkError::Canonicalization("bad".into())), 3);
    assert_eq!(
        exit_code_for(&SdkError::UnsupportedFormat(OutputFormat::Sarif)),
        3
    );
    assert_eq!(
        exit_code_for(&SdkError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "io"
        ))),
        4
    );
    assert_eq!(exit_code_for(&SdkError::NotImplemented("todo")), 1);
}

#[test]
fn missing_repo_root_returns_error() {
    let err = tasc_sdk::path_resolver::resolve_repo_root(std::path::Path::new("/"))
        .expect_err("expected RepoNotFound");
    assert!(matches!(err, SdkError::RepoNotFound(_)));
}

#[test]
fn invalid_format_flag_returns_error() {
    let request = tasc_sdk::DoctorRequest {
        operation: None,
        profile_bundle: None,
        trust_mode: None,
        format: OutputFormat::Sarif,
        timeout_ms: None,
    };
    let err = tasc_sdk::commands::doctor::build_args(&request).expect_err("invalid format");
    assert!(matches!(err, SdkError::UnsupportedFormat(_)));
}
