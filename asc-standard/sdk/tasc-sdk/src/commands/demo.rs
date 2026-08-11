use std::path::PathBuf;

use crate::{DemoRequest, ExplainRequest, VerifyRequest};

pub fn build_demo_requests(request: &DemoRequest) -> (VerifyRequest, ExplainRequest) {
    let verify = VerifyRequest {
        target_path: PathBuf::from("examples/minimal-local"),
        profile_bundle: request.profile_bundle.clone(),
        trust_mode: request.trust_mode.clone(),
        profile: None,
        timeout_ms: request.timeout_ms,
        format: request.format,
    };

    let explain = ExplainRequest {
        report_path: None,
        format: request.format,
        renderer: None,
        timeout_ms: request.timeout_ms,
    };

    (verify, explain)
}
