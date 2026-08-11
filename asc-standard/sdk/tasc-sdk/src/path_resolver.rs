use std::path::{Path, PathBuf};

use crate::SdkError;

pub fn resolve_repo_root(start: &Path) -> Result<PathBuf, SdkError> {
    for ancestor in start.ancestors() {
        if let Some(root) = repo_root_candidate(ancestor) {
            return Ok(root);
        }
    }
    Err(SdkError::RepoNotFound(start.to_path_buf()))
}

pub fn resolve_tasc_path(
    repo_root: &Path,
    override_path: Option<PathBuf>,
) -> Result<PathBuf, SdkError> {
    if let Some(path) = override_path {
        if path.is_file() {
            return Ok(path);
        }
        return Err(SdkError::TascNotFound(path));
    }
    let candidate = repo_root.join("tasc");
    if candidate.is_file() {
        return Ok(candidate);
    }
    Err(SdkError::TascNotFound(candidate))
}

fn repo_root_candidate(base: &Path) -> Option<PathBuf> {
    if is_tasc_root(base) {
        return Some(base.to_path_buf());
    }
    let nested = base.join("TASC").join("asc-standard");
    if is_tasc_root(&nested) {
        return Some(nested);
    }
    None
}

fn is_tasc_root(path: &Path) -> bool {
    path.join("tasc").is_file() && path.join("tasc.yaml").is_file()
}
