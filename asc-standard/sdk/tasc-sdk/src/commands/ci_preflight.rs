use crate::{CiPreflightRequest, OutputFormat, SdkError};

pub fn build_args(request: &CiPreflightRequest) -> Result<Vec<String>, SdkError> {
    if request.format == OutputFormat::Sarif {
        return Err(SdkError::UnsupportedFormat(request.format));
    }

    let mut args = vec!["ci-preflight".to_string()];

    if let Some(examples) = &request.examples {
        args.push("--examples".to_string());
        for example in examples {
            args.push(example.display().to_string());
        }
    }

    args.push("--format".to_string());
    args.push(request.format.as_str().to_string());

    if request.strict {
        args.push("--strict".to_string());
    }

    if let Some(output) = &request.output {
        args.push("--output".to_string());
        args.push(output.display().to_string());
    }
    if let Some(profile_bundle) = &request.profile_bundle {
        args.push("--profile-bundle".to_string());
        args.push(profile_bundle.clone());
    }
    if let Some(trust_mode) = &request.trust_mode {
        args.push("--trust-mode".to_string());
        args.push(trust_mode.clone());
    }

    Ok(args)
}
