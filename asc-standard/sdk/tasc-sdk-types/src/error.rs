use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SdkErrorCode {
    Io,
    RepoNotFound,
    TascNotFound,
    InvalidConfig,
    CommandFailed,
    Timeout,
    ParseError,
    Canonicalization,
    UnsupportedFormat,
    Unknown,
}

impl SdkErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SdkErrorCode::Io => "io",
            SdkErrorCode::RepoNotFound => "repo_not_found",
            SdkErrorCode::TascNotFound => "tasc_not_found",
            SdkErrorCode::InvalidConfig => "invalid_config",
            SdkErrorCode::CommandFailed => "command_failed",
            SdkErrorCode::Timeout => "timeout",
            SdkErrorCode::ParseError => "parse_error",
            SdkErrorCode::Canonicalization => "canonicalization",
            SdkErrorCode::UnsupportedFormat => "unsupported_format",
            SdkErrorCode::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for SdkErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSdkErrorCodeError {
    value: String,
}

impl ParseSdkErrorCodeError {
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl std::fmt::Display for ParseSdkErrorCodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "unsupported error code: {}", self.value)
    }
}

impl std::error::Error for ParseSdkErrorCodeError {}

impl FromStr for SdkErrorCode {
    type Err = ParseSdkErrorCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let normalized = value.trim().to_lowercase().replace('-', "_");
        match normalized.as_str() {
            "io" => Ok(SdkErrorCode::Io),
            "repo_not_found" => Ok(SdkErrorCode::RepoNotFound),
            "tasc_not_found" => Ok(SdkErrorCode::TascNotFound),
            "invalid_config" => Ok(SdkErrorCode::InvalidConfig),
            "command_failed" => Ok(SdkErrorCode::CommandFailed),
            "timeout" => Ok(SdkErrorCode::Timeout),
            "parse_error" => Ok(SdkErrorCode::ParseError),
            "canonicalization" => Ok(SdkErrorCode::Canonicalization),
            "unsupported_format" => Ok(SdkErrorCode::UnsupportedFormat),
            "unknown" => Ok(SdkErrorCode::Unknown),
            other => Err(ParseSdkErrorCodeError {
                value: other.to_string(),
            }),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SdkError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("repo root not found from {0}")]
    RepoNotFound(PathBuf),
    #[error("tasc binary not found at {0}")]
    TascNotFound(PathBuf),
    #[error("invalid config: {0}")]
    InvalidConfig(String),
    #[error("command failed (code={code:?})")]
    CommandFailed { code: Option<i32> },
    #[error("command timed out after {after_ms}ms")]
    Timeout { after_ms: u64 },
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("canonicalization error: {0}")]
    Canonicalization(String),
    #[error("unsupported output format: {0}")]
    UnsupportedFormat(String),
    #[error("unknown error: {0}")]
    Unknown(String),
}

impl SdkError {
    pub fn code(&self) -> SdkErrorCode {
        match self {
            SdkError::Io(_) => SdkErrorCode::Io,
            SdkError::RepoNotFound(_) => SdkErrorCode::RepoNotFound,
            SdkError::TascNotFound(_) => SdkErrorCode::TascNotFound,
            SdkError::InvalidConfig(_) => SdkErrorCode::InvalidConfig,
            SdkError::CommandFailed { .. } => SdkErrorCode::CommandFailed,
            SdkError::Timeout { .. } => SdkErrorCode::Timeout,
            SdkError::ParseError(_) => SdkErrorCode::ParseError,
            SdkError::Canonicalization(_) => SdkErrorCode::Canonicalization,
            SdkError::UnsupportedFormat(_) => SdkErrorCode::UnsupportedFormat,
            SdkError::Unknown(_) => SdkErrorCode::Unknown,
        }
    }
}
