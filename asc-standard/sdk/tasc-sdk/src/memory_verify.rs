use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;

use crate::{verify_path, SdkError, VerifyOptions, VerifyResponse};

pub fn verify_string(input: &str, options: &VerifyOptions) -> Result<VerifyResponse, SdkError> {
    verify_string_with(input, options, verify_path)
}

pub fn verify_bytes(input: &[u8], options: &VerifyOptions) -> Result<VerifyResponse, SdkError> {
    verify_bytes_with(input, options, verify_path)
}

pub fn verify_string_with<F>(
    input: &str,
    options: &VerifyOptions,
    runner: F,
) -> Result<VerifyResponse, SdkError>
where
    F: Fn(&Path, &VerifyOptions) -> Result<VerifyResponse, SdkError>,
{
    verify_bytes_with(input.as_bytes(), options, runner)
}

pub fn verify_bytes_with<F>(
    input: &[u8],
    options: &VerifyOptions,
    runner: F,
) -> Result<VerifyResponse, SdkError>
where
    F: Fn(&Path, &VerifyOptions) -> Result<VerifyResponse, SdkError>,
{
    let mut temp = NamedTempFile::new()?;
    temp.write_all(input)?;
    temp.flush()?;

    let path = temp.path().to_path_buf();
    runner(&path, options)
}
