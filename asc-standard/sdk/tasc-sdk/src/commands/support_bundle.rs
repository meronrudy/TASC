use crate::{SdkError, SupportBundleRequest};

pub fn build_args(request: &SupportBundleRequest) -> Result<Vec<String>, SdkError> {
    let mut args = vec!["support-bundle".to_string()];
    if let Some(path) = &request.target_path {
        args.push(path.display().to_string());
    } else {
        args.push(".".to_string());
    }

    if request.archive {
        args.push("--archive".to_string());
    }
    if let Some(output) = &request.output {
        args.push("--output".to_string());
        args.push(output.display().to_string());
    }
    if request.redact {
        args.push("--redact".to_string());
    }

    Ok(args)
}
