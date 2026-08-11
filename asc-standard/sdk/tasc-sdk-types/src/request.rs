use std::path::PathBuf;

use crate::format::OutputFormat;

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
    pub format: OutputFormat,
    pub timeout_ms: Option<u64>,
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
