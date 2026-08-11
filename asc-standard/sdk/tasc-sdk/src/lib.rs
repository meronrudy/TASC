use std::path::{Path, PathBuf};

pub mod artifact_templates;
pub mod canonicalization;
pub mod client;
pub mod command_runner;
pub mod commands;
pub mod config;
pub mod constructors;
pub mod diagnostics;
pub mod export;
pub mod memory_verify;
pub mod mock_data;
pub mod output;
pub mod path_resolver;
pub mod renderers;
pub mod reports;
#[cfg(feature = "sandbox")]
pub mod sandbox;
pub mod snippets;
pub mod test_helpers;

pub use client::SdkClient;
pub use config::SdkConfig;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, Default,
)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Human,
    Json,
    Sarif,
}

impl OutputFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OutputFormat::Human => "human",
            OutputFormat::Json => "json",
            OutputFormat::Sarif => "sarif",
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
    CommandFailed {
        code: Option<i32>,
        stdout: String,
        stderr: String,
    },
    #[error("command timed out after {after_ms}ms")]
    Timeout {
        after_ms: u64,
        stdout: String,
        stderr: String,
    },
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("canonicalization error: {0}")]
    Canonicalization(String),
    #[error("unsupported output format: {0:?}")]
    UnsupportedFormat(OutputFormat),
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),
}

pub fn exit_code_for(error: &SdkError) -> i32 {
    match error {
        SdkError::InvalidConfig(_) | SdkError::RepoNotFound(_) | SdkError::TascNotFound(_) => 2,
        SdkError::CommandFailed { .. }
        | SdkError::Timeout { .. }
        | SdkError::ParseError(_)
        | SdkError::Canonicalization(_)
        | SdkError::UnsupportedFormat(_) => 3,
        SdkError::Io(_) => 4,
        SdkError::NotImplemented(_) => 1,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VerifyOptions {
    pub profile_bundle: Option<String>,
    pub trust_mode: Option<String>,
    pub profile: Option<String>,
    pub timeout_ms: Option<u64>,
    pub format: OutputFormat,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            profile_bundle: None,
            trust_mode: None,
            profile: None,
            timeout_ms: None,
            format: OutputFormat::Json,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DoctorRequest {
    pub operation: Option<String>,
    pub profile_bundle: Option<String>,
    pub trust_mode: Option<String>,
    pub format: OutputFormat,
    pub timeout_ms: Option<u64>,
}

impl Default for DoctorRequest {
    fn default() -> Self {
        Self {
            operation: None,
            profile_bundle: None,
            trust_mode: None,
            format: OutputFormat::Human,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VerifyRequest {
    pub target_path: PathBuf,
    pub profile_bundle: Option<String>,
    pub trust_mode: Option<String>,
    pub profile: Option<String>,
    pub timeout_ms: Option<u64>,
    pub format: OutputFormat,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExplainRequest {
    pub report_path: Option<PathBuf>,
    pub format: OutputFormat,
    pub renderer: Option<String>,
    pub timeout_ms: Option<u64>,
}

impl Default for ExplainRequest {
    fn default() -> Self {
        Self {
            report_path: None,
            format: OutputFormat::Human,
            renderer: None,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub struct SupportBundleRequest {
    pub target_path: Option<PathBuf>,
    pub archive: bool,
    pub output: Option<PathBuf>,
    pub redact: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CiPreflightRequest {
    pub examples: Option<Vec<PathBuf>>,
    pub format: OutputFormat,
    pub strict: bool,
    pub output: Option<PathBuf>,
    pub profile_bundle: Option<String>,
    pub trust_mode: Option<String>,
    pub timeout_ms: Option<u64>,
}

impl Default for CiPreflightRequest {
    fn default() -> Self {
        Self {
            examples: None,
            format: OutputFormat::Human,
            strict: false,
            output: None,
            profile_bundle: None,
            trust_mode: None,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoRequest {
    pub format: OutputFormat,
    pub profile_bundle: Option<String>,
    pub trust_mode: Option<String>,
    pub timeout_ms: Option<u64>,
}

impl Default for DemoRequest {
    fn default() -> Self {
        Self {
            format: OutputFormat::Human,
            profile_bundle: None,
            trust_mode: None,
            timeout_ms: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DoctorResponse {
    pub result: String,
    pub output: CommandOutput,
    pub json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VerifyResponse {
    pub status: String,
    pub verdict: String,
    pub report_path: Option<PathBuf>,
    pub output: Option<CommandOutput>,
    pub json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ExplainResponse {
    pub result: String,
    pub output: CommandOutput,
    pub json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SupportBundleResponse {
    pub bundle_path: Option<PathBuf>,
    pub output: CommandOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CiPreflightResponse {
    pub result: String,
    pub output: CommandOutput,
    pub json: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DemoResponse {
    pub verify: VerifyResponse,
    pub explain: ExplainResponse,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum SdkResponse {
    Doctor(DoctorResponse),
    Verify(VerifyResponse),
    Explain(ExplainResponse),
    SupportBundle(SupportBundleResponse),
    CiPreflight(CiPreflightResponse),
    Demo(DemoResponse),
}

impl From<VerifyResponse> for SdkResponse {
    fn from(value: VerifyResponse) -> Self {
        Self::Verify(value)
    }
}

impl From<DoctorResponse> for SdkResponse {
    fn from(value: DoctorResponse) -> Self {
        Self::Doctor(value)
    }
}

impl From<ExplainResponse> for SdkResponse {
    fn from(value: ExplainResponse) -> Self {
        Self::Explain(value)
    }
}

impl From<SupportBundleResponse> for SdkResponse {
    fn from(value: SupportBundleResponse) -> Self {
        Self::SupportBundle(value)
    }
}

impl From<CiPreflightResponse> for SdkResponse {
    fn from(value: CiPreflightResponse) -> Self {
        Self::CiPreflight(value)
    }
}

impl From<DemoResponse> for SdkResponse {
    fn from(value: DemoResponse) -> Self {
        Self::Demo(value)
    }
}

pub fn verify_path(path: &Path, options: &VerifyOptions) -> Result<VerifyResponse, SdkError> {
    let client = SdkClient::new(SdkConfig::default())?;
    let request = VerifyRequest {
        target_path: path.to_path_buf(),
        profile_bundle: options.profile_bundle.clone(),
        trust_mode: options.trust_mode.clone(),
        profile: options.profile.clone(),
        timeout_ms: options.timeout_ms,
        format: options.format,
    };
    client.verify(&request)
}
