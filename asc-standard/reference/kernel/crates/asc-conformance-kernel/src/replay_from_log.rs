use anyhow::{anyhow, bail, Context, Result};
use asc_kernel_runtime::Runtime;
use asc_types::model::KernelInput;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct TraceFile {
    #[serde(rename = "schemaVersion")]
    schema_version: String,
    #[serde(rename = "missionId")]
    mission_id: String,
    profile: String,
    ticks: Vec<TraceTick>,
    #[serde(rename = "finalTipHash")]
    final_tip_hash: String,
}

#[derive(Debug, Deserialize)]
struct TraceTick {
    tick: u64,
    #[serde(rename = "kernel_input")]
    kernel_input: KernelInput,
    #[serde(rename = "expected_verdict")]
    expected_verdict: String,
    #[serde(rename = "expected_reasons")]
    expected_reasons: Vec<String>,
    #[serde(rename = "expected_tip_hash")]
    expected_tip_hash: String,
}

#[derive(Debug, Serialize)]
struct Drift {
    tick: u64,
    expected_verdict: String,
    actual_verdict: String,
    expected_reasons: Vec<String>,
    actual_reasons: Vec<String>,
    expected_tip_hash: String,
    actual_tip_hash: String,
}

#[derive(Debug, Serialize)]
struct ReplayReport {
    mission_id: String,
    profile: String,
    schema_version: String,
    result: String,
    drift_count: usize,
    drift: Vec<Drift>,
}

fn parse_args() -> Result<(PathBuf, String, PathBuf, PathBuf)> {
    let mut args = env::args().skip(1);
    let mut repo_root = PathBuf::from(".");
    let mut profile = String::new();
    let mut trace = PathBuf::new();
    let mut output = PathBuf::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?)
            }
            "--profile" => profile = args.next().ok_or_else(|| anyhow!("missing value"))?,
            "--trace" => trace = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?),
            "--output" => output = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?),
            _ => bail!("unknown argument {}", arg),
        }
    }
    if profile.is_empty() || trace.as_os_str().is_empty() || output.as_os_str().is_empty() {
        bail!("usage: --profile <p> --trace <path> --output <path> [--repo-root <path>]");
    }
    Ok((repo_root, profile, trace, output))
}

fn main() -> Result<()> {
    let (repo_root, profile, trace_path, output_path) = parse_args()?;
    let repo_root = repo_root.canonicalize().unwrap_or(repo_root);
    let trace: TraceFile = serde_json::from_str(
        &fs::read_to_string(&trace_path)
            .with_context(|| format!("failed reading trace {}", trace_path.display()))?,
    )
    .with_context(|| format!("invalid trace JSON {}", trace_path.display()))?;

    if trace.profile != profile {
        bail!(
            "trace profile mismatch: trace={}, arg={}",
            trace.profile,
            profile
        );
    }

    let mut runtime = Runtime::from_repo(&repo_root, &profile)?;
    let mut drift: Vec<Drift> = Vec::new();
    for tick in &trace.ticks {
        let output = runtime.evaluate(&tick.kernel_input);
        let actual_verdict = format!("{:?}", output.verdict);
        let actual_reasons = output
            .reasons
            .iter()
            .map(|reason| format!("{:?}", reason))
            .collect::<Vec<_>>();
        let actual_tip_hash = runtime.tip_hash();

        if actual_verdict != tick.expected_verdict
            || actual_reasons != tick.expected_reasons
            || actual_tip_hash != tick.expected_tip_hash
        {
            drift.push(Drift {
                tick: tick.tick,
                expected_verdict: tick.expected_verdict.clone(),
                actual_verdict,
                expected_reasons: tick.expected_reasons.clone(),
                actual_reasons,
                expected_tip_hash: tick.expected_tip_hash.clone(),
                actual_tip_hash,
            });
        }
    }

    if runtime.tip_hash() != trace.final_tip_hash {
        drift.push(Drift {
            tick: trace.ticks.last().map(|t| t.tick).unwrap_or(0),
            expected_verdict: "final-tip-hash".to_string(),
            actual_verdict: "final-tip-hash".to_string(),
            expected_reasons: Vec::new(),
            actual_reasons: Vec::new(),
            expected_tip_hash: trace.final_tip_hash.clone(),
            actual_tip_hash: runtime.tip_hash(),
        });
    }

    let result = if drift.is_empty() { "PASS" } else { "FAIL" };
    let report = ReplayReport {
        mission_id: trace.mission_id,
        profile,
        schema_version: trace.schema_version,
        result: result.to_string(),
        drift_count: drift.len(),
        drift,
    };
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&report)? + "\n")?;
    println!("{}", serde_json::to_string_pretty(&report)?);

    if result == "FAIL" {
        bail!("replay-from-log detected drift");
    }
    Ok(())
}
