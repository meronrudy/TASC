use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::Serialize;

use crate::SdkError;

const DEFAULT_STYLE: &str = r#"
body { font-family: sans-serif; margin: 2rem; color: #111; }
h1 { font-size: 1.4rem; margin-bottom: 1rem; }
pre { background: #f6f8fa; padding: 1rem; border-radius: 6px; }
"#;

pub fn export_json<P: AsRef<Path>, T: Serialize>(path: P, value: &T) -> Result<(), SdkError> {
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, value)
        .map_err(|err| SdkError::ParseError(err.to_string()))?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    Ok(())
}

pub fn export_html<P: AsRef<Path>>(path: P, title: &str, body: &str) -> Result<(), SdkError> {
    let mut file = File::create(path)?;
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>{title}</title><style>{style}</style></head><body><h1>{title}</h1><pre>{body}</pre></body></html>",
        title = title,
        style = DEFAULT_STYLE,
        body = body
    );
    file.write_all(html.as_bytes())?;
    file.sync_all()?;
    Ok(())
}
