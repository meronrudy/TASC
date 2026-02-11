use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub fn read(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed reading {}", path.display()))
}

pub fn write_if_changed(path: &Path, content: &str) -> Result<bool> {
    if path.exists() {
        let current = fs::read_to_string(path)?;
        if current == content {
            return Ok(false);
        }
    }
    fs::write(path, content)?;
    Ok(true)
}
