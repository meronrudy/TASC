use crate::{SdkError, VerifyRequest};

pub fn build_args(request: &VerifyRequest) -> Result<Vec<String>, SdkError> {
    let mut args = vec!["verify".to_string()];
    args.push(request.target_path.display().to_string());

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
