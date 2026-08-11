use tasc_sdk::snippets::{SnippetOperation, SnippetRegistry};
use tasc_sdk::OutputFormat;

#[test]
fn registry_covers_all_operations() {
    let registry = SnippetRegistry::default();
    for operation in SnippetOperation::ALL {
        let snippet = registry
            .generate(operation, OutputFormat::Json)
            .expect("snippet");
        assert!(snippet.contains(operation.as_str()));
    }
}

#[test]
fn snippets_include_format_when_supported() {
    let registry = SnippetRegistry::default();
    let snippet = registry
        .generate(SnippetOperation::Verify, OutputFormat::Sarif)
        .expect("snippet");
    assert!(snippet.contains("--format sarif"));
}
