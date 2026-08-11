use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "tasc-sdk-cli",
    version,
    about = "CLI wrapper for the Rust TASC SDK"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Doctor(DoctorArgs),
    Verify(VerifyArgs),
    Explain(ExplainArgs),
    SupportBundle(SupportBundleArgs),
    CiPreflight(CiPreflightArgs),
    Demo(DemoArgs),
}

#[derive(Args, Debug)]
pub struct DoctorArgs {
    #[arg(long)]
    pub operation: Option<String>,
    #[arg(long)]
    pub profile_bundle: Option<String>,
    #[arg(long)]
    pub trust_mode: Option<String>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct VerifyArgs {
    #[arg(value_name = "PATH")]
    pub path: PathBuf,
    #[arg(long)]
    pub profile_bundle: Option<String>,
    #[arg(long)]
    pub trust_mode: Option<String>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct ExplainArgs {
    #[arg(value_name = "REPORT")]
    pub report_path: Option<PathBuf>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long)]
    pub renderer: Option<String>,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct SupportBundleArgs {
    #[arg(value_name = "PATH")]
    pub path: Option<PathBuf>,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub archive: bool,
    #[arg(long)]
    pub redact: bool,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct CiPreflightArgs {
    #[arg(long, num_args = 1..)]
    pub examples: Option<Vec<PathBuf>>,
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long)]
    pub strict: bool,
    #[arg(long)]
    pub output: Option<PathBuf>,
    #[arg(long)]
    pub profile_bundle: Option<String>,
    #[arg(long)]
    pub trust_mode: Option<String>,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[derive(Args, Debug)]
pub struct DemoArgs {
    #[arg(long)]
    pub format: Option<String>,
    #[arg(long)]
    pub profile_bundle: Option<String>,
    #[arg(long)]
    pub trust_mode: Option<String>,
    #[arg(long)]
    pub timeout_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_verify_with_path() {
        let cli = Cli::try_parse_from([
            "tasc-sdk-cli",
            "verify",
            "examples/minimal-local",
            "--trust-mode",
            "dev-local",
        ])
        .unwrap();
        match cli.command {
            Command::Verify(args) => {
                assert_eq!(args.path, PathBuf::from("examples/minimal-local"));
                assert_eq!(args.trust_mode.as_deref(), Some("dev-local"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn verify_requires_path() {
        let err = Cli::try_parse_from(["tasc-sdk-cli", "verify"]).unwrap_err();
        assert!(err.to_string().contains("PATH"));
    }

    #[test]
    fn parses_doctor_flags() {
        let cli = Cli::try_parse_from([
            "tasc-sdk-cli",
            "doctor",
            "--operation",
            "verify",
            "--format",
            "json",
            "--timeout-ms",
            "1500",
        ])
        .unwrap();
        match cli.command {
            Command::Doctor(args) => {
                assert_eq!(args.operation.as_deref(), Some("verify"));
                assert_eq!(args.format.as_deref(), Some("json"));
                assert_eq!(args.timeout_ms, Some(1500));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_ci_preflight_examples() {
        let cli = Cli::try_parse_from([
            "tasc-sdk-cli",
            "ci-preflight",
            "--examples",
            "examples/minimal-local",
            "examples/minimal-signed",
            "--profile-bundle",
            "dev-local@1.0.0",
            "--trust-mode",
            "dev-local",
            "--strict",
        ])
        .unwrap();
        match cli.command {
            Command::CiPreflight(args) => {
                assert_eq!(
                    args.examples.unwrap(),
                    vec![
                        PathBuf::from("examples/minimal-local"),
                        PathBuf::from("examples/minimal-signed")
                    ]
                );
                assert!(args.strict);
                assert_eq!(args.profile_bundle.as_deref(), Some("dev-local@1.0.0"));
                assert_eq!(args.trust_mode.as_deref(), Some("dev-local"));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }

    #[test]
    fn parses_support_bundle_archive_flag() {
        let cli = Cli::try_parse_from([
            "tasc-sdk-cli",
            "support-bundle",
            "--archive",
            "--redact",
            "--output",
            "bundle.tar.gz",
        ])
        .unwrap();
        match cli.command {
            Command::SupportBundle(args) => {
                assert!(args.archive);
                assert!(args.redact);
                assert_eq!(args.output, Some(PathBuf::from("bundle.tar.gz")));
            }
            other => panic!("unexpected command: {other:?}"),
        }
    }
}
