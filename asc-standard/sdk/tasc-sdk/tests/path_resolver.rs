use std::fs;
use std::path::PathBuf;

use tasc_sdk::path_resolver::{resolve_repo_root, resolve_tasc_path};
use tasc_sdk::SdkError;
use tempfile::TempDir;

fn make_repo_root(dir: &TempDir) -> PathBuf {
    let root = dir.path().to_path_buf();
    fs::write(root.join("tasc"), "stub").expect("write tasc");
    fs::write(root.join("tasc.yaml"), "stub").expect("write tasc.yaml");
    root
}

#[test]
fn resolve_repo_root_finds_direct_root() {
    let dir = TempDir::new().expect("temp dir");
    let root = make_repo_root(&dir);
    let nested = root.join("nested").join("deeper");
    fs::create_dir_all(&nested).expect("nested dirs");

    let found = resolve_repo_root(&nested).expect("repo root");
    assert_eq!(found, root);
}

#[test]
fn resolve_repo_root_finds_nested_tasc_root() {
    let dir = TempDir::new().expect("temp dir");
    let nested_root = dir.path().join("TASC").join("asc-standard");
    fs::create_dir_all(&nested_root).expect("nested root");
    fs::write(nested_root.join("tasc"), "stub").expect("write tasc");
    fs::write(nested_root.join("tasc.yaml"), "stub").expect("write tasc.yaml");

    let found = resolve_repo_root(dir.path()).expect("repo root");
    assert_eq!(found, nested_root);
}

#[test]
fn resolve_tasc_path_prefers_override() {
    let dir = TempDir::new().expect("temp dir");
    let root = make_repo_root(&dir);
    let override_path = root.join("override-tasc");
    fs::write(&override_path, "stub").expect("write override");

    let found = resolve_tasc_path(&root, Some(override_path.clone())).expect("override path");
    assert_eq!(found, override_path);
}

#[test]
fn resolve_tasc_path_errors_on_missing_override() {
    let dir = TempDir::new().expect("temp dir");
    let root = make_repo_root(&dir);
    let override_path = root.join("missing-tasc");

    let err = resolve_tasc_path(&root, Some(override_path.clone())).expect_err("missing override");
    match err {
        SdkError::TascNotFound(path) => assert_eq!(path, override_path),
        other => panic!("unexpected error: {other:?}"),
    }
}
