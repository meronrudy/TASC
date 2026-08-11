use crate::{DoctorRequest, OutputFormat, SdkError};

pub fn build_args(request: &DoctorRequest) -> Result<Vec<String>, SdkError> {
    if request.format == OutputFormat::Sarif {
        return Err(SdkError::UnsupportedFormat(request.format));
    }

    let mut args = vec!["doctor".to_string()];
    if let Some(operation) = &request.operation {
        args.push("--operation".to_string());
        args.push(operation.clone());
    }
    if let Some(profile_bundle) = &request.profile_bundle {
        args.push("--profile-bundle".to_string());
        args.push(profile_bundle.clone());
    }
    if let Some(trust_mode) = &request.trust_mode {
        args.push("--trust-mode".to_string());
        args.push(trust_mode.clone());
    }
    args.push("--format".to_string());
    args.push(request.format.as_str().to_string());

    Ok(args)
}
