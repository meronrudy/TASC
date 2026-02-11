use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("canonical repo root")
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    dir.push(format!("{}-{}-{}", prefix, std::process::id(), stamp));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn run_verify(repo_root: &Path, bundle_path: &Path, profile: &str, output_path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tasc-verify"))
        .current_dir(repo_root)
        .args([
            "verify",
            "--bundle",
            bundle_path
                .to_str()
                .expect("bundle path must be valid UTF-8"),
            "--profile",
            profile,
            "--policy",
            "eu-north-star",
            "--require-ta",
            "TA2",
            "--require-transparency",
            "rekor,mirror",
            "--output",
            output_path
                .to_str()
                .expect("output path must be valid UTF-8"),
        ])
        .output()
        .expect("run tasc-verify")
}

#[test]
fn verify_passes_on_canonical_fixture() {
    let root = repo_root();
    let temp = unique_temp_dir("tasc-verify-pass");
    let bundle = root.join("conformance/fixtures/tasc/tasc-assurance-pack-uas-small.json");
    let output = temp.join("report.json");

    let run = run_verify(&root, &bundle, "uas-small", &output);
    assert!(
        run.status.success(),
        "verify must pass for canonical fixture, stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );

    let report: Value = serde_json::from_str(
        &fs::read_to_string(&output).expect("read report output"),
    )
    .expect("parse report JSON");
    assert_eq!(report.get("result").and_then(Value::as_str), Some("PASS"));
    assert_eq!(
        report
            .get("failedChecks")
            .and_then(Value::as_array)
            .map(std::vec::Vec::len),
        Some(0)
    );
}

#[test]
fn verify_fails_with_specific_check_id_for_retention_policy() {
    let root = repo_root();
    let temp = unique_temp_dir("tasc-verify-fail");
    let source_bundle = root.join("conformance/fixtures/tasc/tasc-assurance-pack-uas-small.json");
    let modified_bundle = temp.join("bundle-retention-fail.json");
    let output = temp.join("report.json");

    let mut bundle: Value = serde_json::from_str(
        &fs::read_to_string(&source_bundle).expect("read fixture bundle"),
    )
    .expect("parse fixture JSON");
    let retention = bundle
        .pointer_mut("/evidenceMap/logs/retentionPolicy/minimumMonths")
        .expect("retention path");
    *retention = Value::from(1_u64);
    fs::write(
        &modified_bundle,
        serde_json::to_string_pretty(&bundle).expect("serialize modified bundle"),
    )
    .expect("write modified bundle");

    let run = run_verify(&root, &modified_bundle, "uas-small", &output);
    assert!(
        !run.status.success(),
        "verify must fail when EU minimum retention is violated"
    );

    let report: Value = serde_json::from_str(
        &fs::read_to_string(&output).expect("read report output"),
    )
    .expect("parse report JSON");
    assert_eq!(report.get("result").and_then(Value::as_str), Some("FAIL"));
    let failed = report
        .get("failedChecks")
        .and_then(Value::as_array)
        .expect("failedChecks array");
    assert!(
        failed
            .iter()
            .any(|item| item.as_str() == Some("CHK_RETENTION_EU_MINIMUM")),
        "expected CHK_RETENTION_EU_MINIMUM in failedChecks, got {:?}",
        failed
    );
}

#[test]
fn verify_passes_from_external_workdir_with_absolute_paths() {
    let root = repo_root();
    let temp = unique_temp_dir("tasc-verify-external-cwd");
    let bundle = root
        .join("conformance/fixtures/tasc/tasc-assurance-pack-fixed-wing.json")
        .canonicalize()
        .expect("canonical bundle path");
    let output = temp.join("report.json");

    let run = Command::new(env!("CARGO_BIN_EXE_tasc-verify"))
        .current_dir(&temp)
        .args([
            "verify",
            "--bundle",
            bundle.to_str().expect("bundle UTF-8"),
            "--profile",
            "fixed-wing",
            "--policy",
            "eu-north-star",
            "--require-ta",
            "TA2",
            "--require-transparency",
            "rekor,mirror",
            "--checks-file",
            root.join("spec/tasc/checks.yaml")
                .to_str()
                .expect("checks file UTF-8"),
            "--trusted-checkpoints",
            root.join("policies/transparency/trusted-log-checkpoints.json")
                .to_str()
                .expect("trusted checkpoints UTF-8"),
            "--badge-registry",
            root.join("policies/badge-registry.json")
                .to_str()
                .expect("badge registry UTF-8"),
            "--attestation-trust-policy",
            root.join("policies/attestation/trust-policy.json")
                .to_str()
                .expect("attestation policy UTF-8"),
            "--trust-roots",
            root.join("policies/attestation/pki/trust-roots.pem")
                .to_str()
                .expect("trust roots UTF-8"),
            "--revocation-snapshot",
            root.join("policies/attestation/revocation-snapshot.json")
                .to_str()
                .expect("revocation snapshot UTF-8"),
            "--freshness-policy",
            root.join("policies/provenance/freshness-policy.yaml")
                .to_str()
                .expect("freshness policy UTF-8"),
            "--transparency-policy",
            root.join("policies/transparency/verification-policy.yaml")
                .to_str()
                .expect("transparency policy UTF-8"),
            "--remediation-file",
            root.join("spec/tasc/remediation.yaml")
                .to_str()
                .expect("remediation file UTF-8"),
            "--output",
            output.to_str().expect("output UTF-8"),
        ])
        .output()
        .expect("run tasc-verify external cwd");

    assert!(
        run.status.success(),
        "verify should pass from external cwd, stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );

    let report: Value =
        serde_json::from_str(&fs::read_to_string(&output).expect("read output report"))
            .expect("parse output report");
    assert_eq!(report.get("result").and_then(Value::as_str), Some("PASS"));
}

#[test]
fn report_subcommand_summarizes_verify_output() {
    let root = repo_root();
    let temp = unique_temp_dir("tasc-verify-report");
    let bundle = root.join("conformance/fixtures/tasc/tasc-assurance-pack-uas-small.json");
    let report_path = temp.join("verify-report.json");

    let verify = run_verify(&root, &bundle, "uas-small", &report_path);
    assert!(
        verify.status.success(),
        "verify must pass before report, stderr={}",
        String::from_utf8_lossy(&verify.stderr)
    );

    let run = Command::new(env!("CARGO_BIN_EXE_tasc-verify"))
        .current_dir(&root)
        .args([
            "report",
            "--input",
            report_path.to_str().expect("report path UTF-8"),
        ])
        .output()
        .expect("run report subcommand");
    assert!(
        run.status.success(),
        "report command failed, stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(stdout.contains("result: PASS"), "unexpected report output: {stdout}");
    assert!(stdout.contains("profile: uas-small"), "unexpected report output: {stdout}");
}

#[test]
fn check_proof_subcommand_validates_fixture_rekor_proof() {
    let root = repo_root();
    let temp = unique_temp_dir("tasc-check-proof");
    let bundle_path = root.join("conformance/fixtures/tasc/tasc-assurance-pack-uas-small.json");
    let bundle: Value =
        serde_json::from_str(&fs::read_to_string(&bundle_path).expect("read fixture bundle"))
            .expect("parse fixture bundle");
    let rekor = bundle
        .pointer("/transparencyProofs/rekor")
        .cloned()
        .expect("rekor proof");

    let proof_path = temp.join("rekor-proof.json");
    fs::write(
        &proof_path,
        serde_json::to_string_pretty(&rekor).expect("serialize proof"),
    )
    .expect("write proof file");

    let run = Command::new(env!("CARGO_BIN_EXE_tasc-verify"))
        .current_dir(&temp)
        .args([
            "check-proof",
            "--proof",
            proof_path.to_str().expect("proof path UTF-8"),
            "--trusted-checkpoints",
            root.join("policies/transparency/trusted-log-checkpoints.json")
                .to_str()
                .expect("trusted checkpoints UTF-8"),
        ])
        .output()
        .expect("run check-proof subcommand");
    assert!(
        run.status.success(),
        "check-proof failed, stderr={}",
        String::from_utf8_lossy(&run.stderr)
    );
    let output: Value = serde_json::from_slice(&run.stdout).expect("parse check-proof output");
    assert_eq!(output.get("result").and_then(Value::as_str), Some("PASS"));
}
