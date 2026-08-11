use std::path::PathBuf;

use crate::{OutputFormat, VerifyRequest};

pub fn minimal_local() -> VerifyRequest {
    VerifyRequest {
        target_path: example_path("minimal-local"),
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        trust_mode: Some("dev-local".to_string()),
        profile: Some("uas-small".to_string()),
        timeout_ms: None,
        format: OutputFormat::Json,
    }
}

pub fn minimal_signed() -> VerifyRequest {
    VerifyRequest {
        target_path: example_path("minimal-signed"),
        profile_bundle: Some("staging-signed@1.0.0".to_string()),
        trust_mode: Some("staging-signed".to_string()),
        profile: Some("fixed-wing".to_string()),
        timeout_ms: None,
        format: OutputFormat::Json,
    }
}

pub fn minimal_replay() -> VerifyRequest {
    VerifyRequest {
        target_path: example_path("minimal-replay"),
        profile_bundle: Some("dev-local@1.0.0".to_string()),
        trust_mode: Some("dev-local".to_string()),
        profile: Some("hybrid-vtol".to_string()),
        timeout_ms: None,
        format: OutputFormat::Json,
    }
}

fn example_path(example: &str) -> PathBuf {
    PathBuf::from("examples").join(example)
}
