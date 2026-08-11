use crate::{
    CiPreflightResponse, DemoResponse, DoctorResponse, ExplainResponse, OutputFormat, SdkError,
    SdkResponse, SupportBundleResponse, VerifyResponse,
};

pub trait Renderer {
    fn render_human(&self, response: &SdkResponse) -> String;
    fn render_json(&self, response: &SdkResponse) -> Result<String, SdkError>;
}

pub struct DefaultRenderer;

impl Renderer for DefaultRenderer {
    fn render_human(&self, response: &SdkResponse) -> String {
        render_human_response(response)
    }

    fn render_json(&self, response: &SdkResponse) -> Result<String, SdkError> {
        JsonRenderer.render_json(response)
    }
}

pub struct JsonRenderer;

impl Renderer for JsonRenderer {
    fn render_human(&self, response: &SdkResponse) -> String {
        render_human_response(response)
    }

    fn render_json(&self, response: &SdkResponse) -> Result<String, SdkError> {
        serde_json::to_string_pretty(response)
            .map_err(|err| SdkError::Canonicalization(err.to_string()))
    }
}

pub fn render_with_format(
    renderer: &dyn Renderer,
    response: &SdkResponse,
    format: OutputFormat,
) -> Result<String, SdkError> {
    match format {
        OutputFormat::Human => Ok(renderer.render_human(response)),
        OutputFormat::Json => renderer.render_json(response),
        OutputFormat::Sarif => Err(SdkError::UnsupportedFormat(format)),
    }
}

fn render_human_response(response: &SdkResponse) -> String {
    match response {
        SdkResponse::Doctor(DoctorResponse { result, .. }) => {
            format!("doctor: {result}")
        }
        SdkResponse::Verify(VerifyResponse {
            status,
            verdict,
            report_path,
            ..
        }) => {
            let report = report_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "none".to_string());
            format!("verify: {status} (verdict={verdict}, report={report})")
        }
        SdkResponse::Explain(ExplainResponse { result, .. }) => {
            format!("explain: {result}")
        }
        SdkResponse::SupportBundle(SupportBundleResponse { bundle_path, .. }) => {
            let path = bundle_path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "none".to_string());
            format!("support-bundle: {path}")
        }
        SdkResponse::CiPreflight(CiPreflightResponse { result, .. }) => {
            format!("ci-preflight: {result}")
        }
        SdkResponse::Demo(DemoResponse { verify, explain }) => {
            format!("demo: verify={} explain={}", verify.status, explain.result)
        }
    }
}
