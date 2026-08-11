use std::str::FromStr;

use tasc_sdk_types::OutputFormat;

#[test]
fn parses_output_formats() {
    assert_eq!(
        OutputFormat::from_str("human").unwrap(),
        OutputFormat::Human
    );
    assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
    assert_eq!(
        OutputFormat::from_str("sarif").unwrap(),
        OutputFormat::Sarif
    );
    assert_eq!(
        OutputFormat::from_str(" JSON ").unwrap(),
        OutputFormat::Json
    );
}

#[test]
fn rejects_invalid_format() {
    let err = OutputFormat::from_str("xml").expect_err("should fail");
    assert!(err.to_string().contains("unsupported"));
    assert_eq!(err.value(), "xml");
}

#[test]
fn displays_and_serializes_formats() {
    assert_eq!(OutputFormat::Human.as_str(), "human");
    assert_eq!(OutputFormat::Human.to_string(), "human");

    let json = serde_json::to_string(&OutputFormat::Sarif).unwrap();
    assert_eq!(json, "\"sarif\"");
    let decoded: OutputFormat = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, OutputFormat::Sarif);
}
