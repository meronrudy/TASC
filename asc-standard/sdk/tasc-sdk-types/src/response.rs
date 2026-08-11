use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
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
