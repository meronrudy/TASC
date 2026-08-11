use tasc_sdk::renderers::{render_with_format, DefaultRenderer, JsonRenderer, Renderer};
use tasc_sdk::{OutputFormat, SdkResponse, VerifyResponse};

fn sample_response() -> SdkResponse {
    SdkResponse::Verify(VerifyResponse {
        status: "PASS".to_string(),
        verdict: "OK".to_string(),
        report_path: None,
        output: None,
        json: None,
    })
}

#[test]
fn default_renderer_human_output() {
    let response = sample_response();
    let renderer = DefaultRenderer;

    let output = renderer.render_human(&response);
    assert!(output.contains("verify:"));
    assert!(output.contains("verdict=OK"));
}

#[test]
fn json_renderer_outputs_json() {
    let response = sample_response();
    let renderer = JsonRenderer;

    let output = renderer
        .render_json(&response)
        .expect("json render should succeed");
    let parsed: serde_json::Value = serde_json::from_str(&output).expect("valid json");
    assert_eq!(parsed["type"], "Verify");
    assert_eq!(parsed["data"]["verdict"], "OK");
}

#[test]
fn render_with_format_routes_output() {
    let response = sample_response();
    let renderer = DefaultRenderer;

    let human =
        render_with_format(&renderer, &response, OutputFormat::Human).expect("human output");
    assert!(human.contains("verify:"));

    let json = render_with_format(&renderer, &response, OutputFormat::Json).expect("json output");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
    assert_eq!(parsed["data"]["status"], "PASS");
}

#[test]
fn sarif_is_not_supported() {
    let response = sample_response();
    let renderer = DefaultRenderer;

    let err = render_with_format(&renderer, &response, OutputFormat::Sarif)
        .expect_err("sarif should fail");
    assert!(err.to_string().contains("unsupported"));
}
