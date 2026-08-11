use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use crate::command_runner::CommandResult;
use crate::commands;
use crate::config::SdkConfig;
use crate::diagnostics::{self, DiagnosticReport};
use crate::path_resolver;
use crate::{
    CiPreflightRequest, CiPreflightResponse, CommandOutput, DemoRequest, DemoResponse,
    DoctorRequest, DoctorResponse, ExplainRequest, ExplainResponse, SdkError, SupportBundleRequest,
    SupportBundleResponse, VerifyRequest, VerifyResponse,
};

pub struct SdkClient {
    config: SdkConfig,
    repo_root: PathBuf,
    tasc_path: PathBuf,
}

impl SdkClient {
    pub fn new(config: SdkConfig) -> Result<Self, SdkError> {
        let cwd = std::env::current_dir()?;
        let repo_root = if let Some(root) = config.repo_root.clone() {
            root
        } else {
            path_resolver::resolve_repo_root(&cwd)?
        };
        let tasc_path = path_resolver::resolve_tasc_path(&repo_root, config.tasc_path.clone())?;

        Ok(Self {
            config,
            repo_root,
            tasc_path,
        })
    }

    pub fn doctor(&self, request: &DoctorRequest) -> Result<DoctorResponse, SdkError> {
        let args = commands::doctor::build_args(request)?;
        let result = self.run_tasc(args, request.timeout_ms, true)?;
        let output = Self::command_output(&result);
        let json = Self::parse_json(&output.stdout);
        let result_value = Self::result_from_json(json.as_ref())
            .or_else(|| Self::result_from_output(&output))
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Ok(DoctorResponse {
            result: result_value,
            output,
            json,
        })
    }

    pub fn verify(&self, request: &VerifyRequest) -> Result<VerifyResponse, SdkError> {
        let args = commands::verify::build_args(request)?;
        let result = self.run_tasc(args, request.timeout_ms, true)?;
        let output = Self::command_output(&result);
        let json = Self::parse_json(&output.stdout);
        let verdict = Self::result_from_json(json.as_ref())
            .or_else(|| Self::result_from_output(&output))
            .unwrap_or_else(|| "UNKNOWN".to_string());
        let report_path = self.repo_root.join(".tasc/last-report.json");

        Ok(VerifyResponse {
            status: verdict.clone(),
            verdict,
            report_path: report_path.exists().then_some(report_path),
            output: Some(output),
            json,
        })
    }

    pub fn explain(&self, request: &ExplainRequest) -> Result<ExplainResponse, SdkError> {
        let args = commands::explain::build_args(request)?;
        let result = self.run_tasc(args, request.timeout_ms, true)?;
        let output = Self::command_output(&result);
        let json = Self::parse_json(&output.stdout);
        let result_value = Self::result_from_json(json.as_ref())
            .or_else(|| Self::result_from_output(&output))
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Ok(ExplainResponse {
            result: result_value,
            output,
            json,
        })
    }

    pub fn support_bundle(
        &self,
        request: &SupportBundleRequest,
    ) -> Result<SupportBundleResponse, SdkError> {
        let args = commands::support_bundle::build_args(request)?;
        let result = self.run_tasc(args, request.timeout_ms, false)?;
        let output = Self::command_output(&result);
        let bundle_path = output.stdout.trim();
        let resolved = if bundle_path.is_empty() {
            None
        } else {
            Some(PathBuf::from(bundle_path))
        };

        Ok(SupportBundleResponse {
            bundle_path: resolved,
            output,
        })
    }

    pub fn ci_preflight(
        &self,
        request: &CiPreflightRequest,
    ) -> Result<CiPreflightResponse, SdkError> {
        let args = commands::ci_preflight::build_args(request)?;
        let result = self.run_tasc(args, request.timeout_ms, true)?;
        let output = Self::command_output(&result);
        let json = Self::parse_json(&output.stdout);
        let result_value = Self::result_from_json(json.as_ref())
            .or_else(|| Self::result_from_output(&output))
            .unwrap_or_else(|| "UNKNOWN".to_string());

        Ok(CiPreflightResponse {
            result: result_value,
            output,
            json,
        })
    }

    pub fn demo(&self, request: &DemoRequest) -> Result<DemoResponse, SdkError> {
        let (verify_request, explain_request) = commands::demo::build_demo_requests(request);
        let verify = self.verify(&verify_request)?;
        let explain = self.explain(&explain_request)?;

        Ok(DemoResponse { verify, explain })
    }

    pub fn diagnostics(&self) -> Result<DiagnosticReport, SdkError> {
        diagnostics::run(&self.config, Some(&self.repo_root))
    }

    fn run_tasc(
        &self,
        args: Vec<String>,
        timeout_ms: Option<u64>,
        allow_failure: bool,
    ) -> Result<CommandResult, SdkError> {
        let mut cmd = Command::new(&self.tasc_path);
        cmd.current_dir(&self.repo_root);
        cmd.args(args);

        if allow_failure {
            crate::command_runner::run_allow_failure(&mut cmd, self.resolve_timeout(timeout_ms))
        } else {
            crate::command_runner::run(&mut cmd, self.resolve_timeout(timeout_ms))
        }
    }

    fn resolve_timeout(&self, request_timeout: Option<u64>) -> Option<Duration> {
        request_timeout
            .or(self.config.timeout_ms)
            .map(Duration::from_millis)
    }

    fn command_output(result: &CommandResult) -> CommandOutput {
        CommandOutput {
            stdout: result.stdout.clone(),
            stderr: result.stderr.clone(),
            exit_code: result.status.code(),
        }
    }

    fn parse_json(stdout: &str) -> Option<serde_json::Value> {
        let trimmed = stdout.trim();
        if trimmed.is_empty() {
            return None;
        }
        serde_json::from_str(trimmed).ok()
    }

    fn result_from_json(json: Option<&serde_json::Value>) -> Option<String> {
        json.and_then(|value| {
            value
                .get("result")
                .or_else(|| value.get("status"))
                .or_else(|| value.get("verdict"))
                .and_then(|result| result.as_str())
                .map(|result| result.to_string())
        })
    }

    fn result_from_output(output: &CommandOutput) -> Option<String> {
        let combined = format!("{} {}", output.stdout, output.stderr);
        let upper = combined.to_uppercase();
        if upper.contains("FAIL") {
            Some("FAIL".to_string())
        } else if upper.contains("WARN") {
            Some("WARN".to_string())
        } else if upper.contains("PASS") {
            Some("PASS".to_string())
        } else {
            None
        }
    }
}
