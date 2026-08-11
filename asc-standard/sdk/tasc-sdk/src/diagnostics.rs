use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use crate::config::SdkConfig;
use crate::path_resolver;
use crate::SdkError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticFinding {
    pub code: String,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub remediation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticReport {
    pub result: String,
    pub findings: Vec<DiagnosticFinding>,
    pub context: BTreeMap<String, String>,
}

pub fn run(
    config: &SdkConfig,
    workspace_hint: Option<&Path>,
) -> Result<DiagnosticReport, SdkError> {
    let mut findings = Vec::new();
    let mut context = BTreeMap::new();

    let start_path = if let Some(path) = &config.repo_root {
        path.clone()
    } else if let Some(path) = workspace_hint {
        path.to_path_buf()
    } else {
        std::env::current_dir()?
    };

    context.insert("startPath".to_string(), start_path.display().to_string());

    let repo_root = match path_resolver::resolve_repo_root(&start_path) {
        Ok(path) => {
            context.insert("repoRoot".to_string(), path.display().to_string());
            Some(path)
        }
        Err(SdkError::RepoNotFound(_)) => {
            findings.push(DiagnosticFinding {
                code: "REPO_ROOT_NOT_FOUND".to_string(),
                severity: DiagnosticSeverity::Error,
                message: format!("could not resolve repo root from {}", start_path.display()),
                remediation: "run from inside the repository or set SdkConfig.repo_root"
                    .to_string(),
            });
            None
        }
        Err(err) => return Err(err),
    };

    if let Some(root) = repo_root.as_ref() {
        check_tasc_path(config, root, &mut findings, &mut context)?;
        check_examples(root, &mut findings);
    }

    check_tool(
        "rustc",
        "--version",
        "RUSTC_NOT_AVAILABLE",
        &mut findings,
        &mut context,
    );
    check_tool(
        "cargo",
        "--version",
        "CARGO_NOT_AVAILABLE",
        &mut findings,
        &mut context,
    );

    let has_error = findings
        .iter()
        .any(|finding| finding.severity == DiagnosticSeverity::Error);
    let has_warn = findings
        .iter()
        .any(|finding| finding.severity == DiagnosticSeverity::Warn);

    let result = if has_error {
        "FAIL"
    } else if has_warn {
        "WARN"
    } else {
        "PASS"
    };

    Ok(DiagnosticReport {
        result: result.to_string(),
        findings,
        context,
    })
}

pub fn is_healthy(report: &DiagnosticReport) -> bool {
    report.result == "PASS"
        && !report
            .findings
            .iter()
            .any(|finding| finding.severity == DiagnosticSeverity::Error)
}

fn check_tasc_path(
    config: &SdkConfig,
    repo_root: &Path,
    findings: &mut Vec<DiagnosticFinding>,
    context: &mut BTreeMap<String, String>,
) -> Result<(), SdkError> {
    match path_resolver::resolve_tasc_path(repo_root, config.tasc_path.clone()) {
        Ok(path) => {
            context.insert("tascPath".to_string(), path.display().to_string());
            if !is_executable(&path) {
                findings.push(DiagnosticFinding {
                    code: "TASC_NOT_EXECUTABLE".to_string(),
                    severity: DiagnosticSeverity::Error,
                    message: format!("resolved tasc path is not executable: {}", path.display()),
                    remediation: "ensure `tasc` has executable permissions".to_string(),
                });
            }
        }
        Err(SdkError::TascNotFound(path)) => {
            findings.push(DiagnosticFinding {
                code: "TASC_NOT_FOUND".to_string(),
                severity: DiagnosticSeverity::Error,
                message: format!("could not resolve tasc executable at {}", path.display()),
                remediation: "set SdkConfig.tasc_path or restore repository wrapper binary"
                    .to_string(),
            });
        }
        Err(err) => return Err(err),
    }

    Ok(())
}

fn check_examples(repo_root: &Path, findings: &mut Vec<DiagnosticFinding>) {
    for example in ["minimal-local", "minimal-signed", "minimal-replay"] {
        let path = repo_root.join("examples").join(example);
        if !path.is_dir() {
            findings.push(DiagnosticFinding {
                code: format!(
                    "MISSING_EXAMPLE_{}",
                    example.replace('-', "_").to_uppercase()
                ),
                severity: DiagnosticSeverity::Error,
                message: format!("required example is missing: {}", path.display()),
                remediation: "restore repository examples directory".to_string(),
            });
        }
    }
}

fn check_tool(
    tool: &str,
    arg: &str,
    error_code: &str,
    findings: &mut Vec<DiagnosticFinding>,
    context: &mut BTreeMap<String, String>,
) {
    match Command::new(tool).arg(arg).output() {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
            context.insert(format!("{}Version", tool), version);
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            findings.push(DiagnosticFinding {
                code: error_code.to_string(),
                severity: DiagnosticSeverity::Error,
                message: format!("{tool} reported non-zero status: {stderr}"),
                remediation: format!("reinstall or repair {tool} toolchain"),
            });
        }
        Err(err) => {
            findings.push(DiagnosticFinding {
                code: error_code.to_string(),
                severity: DiagnosticSeverity::Error,
                message: format!("failed to execute {tool}: {err}"),
                remediation: format!("ensure `{tool}` is installed and available on PATH"),
            });
        }
    }
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .map(|metadata| metadata.is_file() && (metadata.permissions().mode() & 0o111) != 0)
        .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}
