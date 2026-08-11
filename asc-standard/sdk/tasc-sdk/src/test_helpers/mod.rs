use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::{ExplainRequest, OutputFormat, SdkClient, SdkError, VerifyRequest};

pub fn seeded_bundle_bytes() -> Vec<u8> {
    br#"{"bundle":"seeded","version":1,"entries":[{"id":"alpha","value":true}]}"#.to_vec()
}

pub fn seeded_bundle_file() -> Result<NamedTempFile, SdkError> {
    let mut file = NamedTempFile::new()?;
    file.write_all(&seeded_bundle_bytes())?;
    file.flush()?;
    Ok(file)
}

pub fn assert_verify_and_explain(
    client: &SdkClient,
    path: &Path,
) -> Result<(crate::VerifyResponse, crate::ExplainResponse), SdkError> {
    let verify_request = VerifyRequest {
        target_path: path.to_path_buf(),
        profile_bundle: None,
        trust_mode: None,
        profile: None,
        timeout_ms: None,
        format: OutputFormat::Json,
    };
    let verify = client.verify(&verify_request)?;
    let report_path = verify
        .report_path
        .clone()
        .ok_or_else(|| SdkError::ParseError("verify response missing report path".to_string()))?;
    let explain_request = ExplainRequest {
        report_path: Some(report_path),
        format: OutputFormat::Json,
        renderer: None,
        timeout_ms: None,
    };
    let explain = client.explain(&explain_request)?;
    Ok((verify, explain))
}
