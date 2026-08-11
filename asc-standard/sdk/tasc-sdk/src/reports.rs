use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::SdkError;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LastArtifacts {
    pub report_path: Option<PathBuf>,
    pub explain_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ReportSummary {
    pub result: String,
    pub verdict: String,
    pub findings_count: usize,
}

pub fn locate(repo_root: &Path) -> LastArtifacts {
    let state_dir = repo_root.join(".tasc");
    let report = state_dir.join("last-report.json");
    let explain = state_dir.join("last-explain.json");

    LastArtifacts {
        report_path: report.exists().then_some(report),
        explain_path: explain.exists().then_some(explain),
    }
}

pub fn read_last_report(repo_root: &Path) -> Result<Value, SdkError> {
    let paths = locate(repo_root);
    let Some(path) = paths.report_path else {
        return Err(SdkError::ParseError("last report not found".to_string()));
    };
    read_json_file(&path)
}

pub fn read_last_explain(repo_root: &Path) -> Result<Value, SdkError> {
    let paths = locate(repo_root);
    let Some(path) = paths.explain_path else {
        return Err(SdkError::ParseError("last explain not found".to_string()));
    };
    read_json_file(&path)
}

pub fn summarize(report: &Value) -> ReportSummary {
    let result = report
        .get("result")
        .or_else(|| report.get("status"))
        .and_then(|value| value.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();

    let verdict = report
        .get("verdict")
        .or_else(|| report.get("result"))
        .or_else(|| report.get("status"))
        .and_then(|value| value.as_str())
        .unwrap_or("UNKNOWN")
        .to_string();

    let findings_count = report
        .get("findings")
        .and_then(|value| value.as_array())
        .map(|array| array.len())
        .or_else(|| {
            report
                .get("failures")
                .and_then(|value| value.as_array())
                .map(|array| array.len())
        })
        .unwrap_or(0);

    ReportSummary {
        result,
        verdict,
        findings_count,
    }
}

fn read_json_file(path: &Path) -> Result<Value, SdkError> {
    let bytes = fs::read(path)?;
    serde_json::from_slice::<Value>(&bytes).map_err(|err| SdkError::ParseError(err.to_string()))
}
