use crate::{ExplainRequest, SdkError};

pub fn build_args(request: &ExplainRequest) -> Result<Vec<String>, SdkError> {
    let mut args = vec!["explain".to_string()];
    if let Some(report_path) = &request.report_path {
        args.push(report_path.display().to_string());
    } else {
        args.push("last".to_string());
    }

    args.push("--format".to_string());
    args.push(request.format.as_str().to_string());

    if let Some(renderer) = &request.renderer {
        args.push("--renderer".to_string());
        args.push(renderer.clone());
    }

    Ok(args)
}
