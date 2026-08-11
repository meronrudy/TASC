use std::collections::HashMap;

use crate::{OutputFormat, SdkError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SnippetOperation {
    Doctor,
    Verify,
    Explain,
    SupportBundle,
    CiPreflight,
    Demo,
}

impl SnippetOperation {
    pub const ALL: [SnippetOperation; 6] = [
        SnippetOperation::Doctor,
        SnippetOperation::Verify,
        SnippetOperation::Explain,
        SnippetOperation::SupportBundle,
        SnippetOperation::CiPreflight,
        SnippetOperation::Demo,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            SnippetOperation::Doctor => "doctor",
            SnippetOperation::Verify => "verify",
            SnippetOperation::Explain => "explain",
            SnippetOperation::SupportBundle => "support-bundle",
            SnippetOperation::CiPreflight => "ci-preflight",
            SnippetOperation::Demo => "demo",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SnippetRegistry {
    entries: HashMap<(SnippetOperation, OutputFormat), String>,
}

impl SnippetRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn insert(&mut self, operation: SnippetOperation, format: OutputFormat, snippet: String) {
        self.entries.insert((operation, format), snippet);
    }

    pub fn snippet(
        &self,
        operation: SnippetOperation,
        format: OutputFormat,
    ) -> Result<&str, SdkError> {
        self.entries
            .get(&(operation, format))
            .map(|value| value.as_str())
            .ok_or(SdkError::NotImplemented("snippet not available"))
    }

    pub fn generate(
        &self,
        operation: SnippetOperation,
        format: OutputFormat,
    ) -> Result<String, SdkError> {
        Ok(self.snippet(operation, format)?.to_string())
    }
}

impl Default for SnippetRegistry {
    fn default() -> Self {
        let mut registry = SnippetRegistry::new();
        for operation in SnippetOperation::ALL {
            for format in [OutputFormat::Human, OutputFormat::Json, OutputFormat::Sarif] {
                let snippet = default_snippet(operation, format);
                registry.insert(operation, format, snippet);
            }
        }
        registry
    }
}

fn default_snippet(operation: SnippetOperation, format: OutputFormat) -> String {
    let fmt = format.as_str();
    match operation {
        SnippetOperation::Doctor => {
            format!("tasc-sdk-cli doctor --operation verify --format {fmt}")
        }
        SnippetOperation::Verify => {
            format!("tasc-sdk-cli verify examples/minimal-local --format {fmt}")
        }
        SnippetOperation::Explain => format!("tasc-sdk-cli explain last --format {fmt}"),
        SnippetOperation::SupportBundle => "tasc-sdk-cli support-bundle --archive".to_string(),
        SnippetOperation::CiPreflight => {
            format!("tasc-sdk-cli ci-preflight --strict --format {fmt}")
        }
        SnippetOperation::Demo => format!("tasc-sdk-cli demo --format {fmt}"),
    }
}
