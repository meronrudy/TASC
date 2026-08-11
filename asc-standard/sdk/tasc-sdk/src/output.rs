use crate::renderers::{render_with_format, DefaultRenderer, Renderer};
use crate::{OutputFormat, SdkError, SdkResponse};

pub fn render_response(response: &SdkResponse, format: OutputFormat) -> Result<String, SdkError> {
    let renderer = DefaultRenderer;
    render_with_format(&renderer, response, format)
}

pub fn render_response_with(
    renderer: &dyn Renderer,
    response: &SdkResponse,
    format: OutputFormat,
) -> Result<String, SdkError> {
    render_with_format(renderer, response, format)
}
