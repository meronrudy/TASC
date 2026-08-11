use std::path::PathBuf;
use std::str::FromStr;

use tasc_sdk_types::{SdkError, SdkErrorCode};

#[test]
fn maps_error_codes() {
    let errors = vec![
        (
            SdkError::Io(std::io::Error::new(std::io::ErrorKind::Other, "io")),
            SdkErrorCode::Io,
        ),
        (
            SdkError::RepoNotFound(PathBuf::from("/")),
            SdkErrorCode::RepoNotFound,
        ),
        (
            SdkError::TascNotFound(PathBuf::from("/tasc")),
            SdkErrorCode::TascNotFound,
        ),
        (
            SdkError::InvalidConfig("bad".into()),
            SdkErrorCode::InvalidConfig,
        ),
        (
            SdkError::CommandFailed { code: Some(2) },
            SdkErrorCode::CommandFailed,
        ),
        (SdkError::Timeout { after_ms: 1000 }, SdkErrorCode::Timeout),
        (
            SdkError::ParseError("parse".into()),
            SdkErrorCode::ParseError,
        ),
        (
            SdkError::Canonicalization("canon".into()),
            SdkErrorCode::Canonicalization,
        ),
        (
            SdkError::UnsupportedFormat("sarif".into()),
            SdkErrorCode::UnsupportedFormat,
        ),
        (SdkError::Unknown("unknown".into()), SdkErrorCode::Unknown),
    ];

    for (error, expected) in errors {
        assert_eq!(error.code(), expected);
    }
}

#[test]
fn parses_error_codes() {
    assert_eq!(SdkErrorCode::from_str("io").unwrap(), SdkErrorCode::Io);
    assert_eq!(
        SdkErrorCode::from_str(" REPO_NOT_FOUND ").unwrap(),
        SdkErrorCode::RepoNotFound
    );
    assert_eq!(
        SdkErrorCode::from_str("repo-not-found").unwrap(),
        SdkErrorCode::RepoNotFound
    );
}

#[test]
fn rejects_invalid_error_code() {
    let err = SdkErrorCode::from_str("not-a-code").expect_err("should fail");
    assert!(err.to_string().contains("unsupported"));
    assert_eq!(err.value(), "not_a_code");
}

#[test]
fn displays_and_serializes_error_codes() {
    assert_eq!(SdkErrorCode::Timeout.as_str(), "timeout");
    assert_eq!(SdkErrorCode::Timeout.to_string(), "timeout");

    let json = serde_json::to_string(&SdkErrorCode::CommandFailed).unwrap();
    assert_eq!(json, "\"command_failed\"");
    let decoded: SdkErrorCode = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, SdkErrorCode::CommandFailed);
}
