use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct SdkConfig {
    pub repo_root: Option<PathBuf>,
    pub tasc_path: Option<PathBuf>,
    pub timeout_ms: Option<u64>,
}
