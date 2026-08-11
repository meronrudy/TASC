mod args;
mod output;

use std::str::FromStr;

use args::{
    CiPreflightArgs, Cli, Command, DemoArgs, DoctorArgs, ExplainArgs, SupportBundleArgs, VerifyArgs,
};
use clap::Parser;
use tasc_sdk::{
    CiPreflightRequest, DemoRequest, DoctorRequest, ExplainRequest, OutputFormat, SdkClient,
    SdkConfig, SdkError, SdkResponse, SupportBundleRequest, VerifyRequest,
};
use tasc_sdk_types::OutputFormat as TypesOutputFormat;

fn main() {
    let cli = Cli::parse();
    let client = match SdkClient::new(SdkConfig::default()) {
        Ok(client) => client,
        Err(err) => exit_with_error(err),
    };

    let result = match cli.command {
        Command::Doctor(args) => run_doctor(&client, args),
        Command::Verify(args) => run_verify(&client, args),
        Command::Explain(args) => run_explain(&client, args),
        Command::SupportBundle(args) => run_support_bundle(&client, args),
        Command::CiPreflight(args) => run_ci_preflight(&client, args),
        Command::Demo(args) => run_demo(&client, args),
    };

    let (response, format) = match result {
        Ok(value) => value,
        Err(err) => exit_with_error(err),
    };

    let rendered = match output::render_response(&response, format) {
        Ok(value) => value,
        Err(err) => exit_with_error(err),
    };
    println!("{rendered}");
}

fn run_doctor(
    client: &SdkClient,
    args: DoctorArgs,
) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let format = resolve_format(args.format)?;
    let request = DoctorRequest {
        operation: args.operation,
        profile_bundle: args.profile_bundle,
        trust_mode: args.trust_mode,
        format,
        timeout_ms: resolve_timeout(args.timeout_ms),
    };
    let response = client.doctor(&request)?;
    Ok((response.into(), format))
}

fn run_verify(
    client: &SdkClient,
    args: VerifyArgs,
) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let format = resolve_format(args.format)?;
    let request = VerifyRequest {
        target_path: args.path,
        profile_bundle: args.profile_bundle,
        trust_mode: args.trust_mode,
        profile: None,
        timeout_ms: resolve_timeout(args.timeout_ms),
        format,
    };
    let response = client.verify(&request)?;
    Ok((response.into(), format))
}

fn run_explain(
    client: &SdkClient,
    args: ExplainArgs,
) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let format = resolve_format(args.format)?;
    let request = ExplainRequest {
        report_path: args.report_path,
        format,
        renderer: args.renderer,
        timeout_ms: resolve_timeout(args.timeout_ms),
    };
    let response = client.explain(&request)?;
    Ok((response.into(), format))
}

fn run_support_bundle(
    client: &SdkClient,
    args: SupportBundleArgs,
) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let request = SupportBundleRequest {
        target_path: args.path,
        archive: args.archive,
        output: args.output,
        redact: args.redact,
        timeout_ms: resolve_timeout(args.timeout_ms),
    };
    let response = client.support_bundle(&request)?;
    Ok((response.into(), OutputFormat::Human))
}

fn run_ci_preflight(
    client: &SdkClient,
    args: CiPreflightArgs,
) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let format = resolve_format(args.format)?;
    let request = CiPreflightRequest {
        examples: args.examples,
        format,
        strict: args.strict,
        output: args.output,
        profile_bundle: args.profile_bundle,
        trust_mode: args.trust_mode,
        timeout_ms: resolve_timeout(args.timeout_ms),
    };
    let response = client.ci_preflight(&request)?;
    Ok((response.into(), format))
}

fn run_demo(client: &SdkClient, args: DemoArgs) -> Result<(SdkResponse, OutputFormat), SdkError> {
    let format = resolve_format(args.format)?;
    let request = DemoRequest {
        format,
        profile_bundle: args.profile_bundle,
        trust_mode: args.trust_mode,
        timeout_ms: resolve_timeout(args.timeout_ms),
    };
    let response = client.demo(&request)?;
    Ok((response.into(), format))
}

fn resolve_format(value: Option<String>) -> Result<OutputFormat, SdkError> {
    match value {
        Some(raw) => {
            let parsed = TypesOutputFormat::from_str(&raw)
                .map_err(|err| SdkError::ParseError(err.to_string()))?;
            Ok(match parsed {
                TypesOutputFormat::Human => OutputFormat::Human,
                TypesOutputFormat::Json => OutputFormat::Json,
                TypesOutputFormat::Sarif => OutputFormat::Sarif,
            })
        }
        None => Ok(OutputFormat::Human),
    }
}

fn resolve_timeout(timeout_ms: Option<u64>) -> Option<u64> {
    timeout_ms.or(default_timeout_ms())
}

fn default_timeout_ms() -> Option<u64> {
    tasc_sdk_types::DoctorRequest::default().timeout_ms
}

fn exit_with_error(err: SdkError) -> ! {
    eprintln!("{err}");
    std::process::exit(tasc_sdk::exit_code_for(&err));
}
