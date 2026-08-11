use tasc_sdk::{OutputFormat, SdkError, SdkResponse};

pub fn render_response(response: &SdkResponse, format: OutputFormat) -> Result<String, SdkError> {
    tasc_sdk::output::render_response(response, format)
}
