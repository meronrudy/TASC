use anyhow::{anyhow, bail, Context, Result};
use asc_kernel_runtime::Runtime;
use asc_types::model::KernelInput;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct KernelVector {
    profile: String,
    input: KernelInput,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct ReplayFixture {
    fixture_id: String,
    description: String,
    profile: String,
    seed: u64,
    ticks: u64,
    time_step_ms: u64,
    #[serde(default)]
    expected_final_verdict: String,
    #[serde(default)]
    expected_tip_hash: String,
    #[serde(default)]
    drift: DriftPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct DriftPolicy {
    #[serde(default = "default_seed_offset")]
    seed_offset: u64,
    #[serde(default = "default_true")]
    require_tip_hash_change: bool,
    #[serde(default = "default_true")]
    require_verdict_stable: bool,
}

impl Default for DriftPolicy {
    fn default() -> Self {
        Self {
            seed_offset: default_seed_offset(),
            require_tip_hash_change: default_true(),
            require_verdict_stable: default_true(),
        }
    }
}

fn default_seed_offset() -> u64 {
    1
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
struct ProfileReplayReport {
    profile: String,
    fixture_id: String,
    baseline_tip_hash: String,
    baseline_final_verdict: String,
    repeated_tip_hash: String,
    repeated_final_verdict: String,
    drift_tip_hash: String,
    drift_final_verdict: String,
    expected_tip_hash: String,
    expected_final_verdict: String,
    result: String,
    failures: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ReplayDriftReport {
    validator: String,
    profiles: Vec<ProfileReplayReport>,
    result: String,
}

#[derive(Debug)]
struct SimResult {
    final_verdict: String,
    tip_hash: String,
}

#[derive(Debug)]
struct Args {
    repo_root: PathBuf,
    profiles: Vec<String>,
    vectors_dir: PathBuf,
    fixtures_dir: PathBuf,
    output: PathBuf,
    update_fixtures: bool,
}

fn parse_args() -> Result<Args> {
    parse_args_from(env::args().skip(1))
}

fn parse_args_from<I>(mut args: I) -> Result<Args>
where
    I: Iterator<Item = String>,
{
    let mut repo_root = PathBuf::from(".");
    let mut profiles = vec![
        "uas-small".to_string(),
        "fixed-wing".to_string(),
        "hybrid-vtol".to_string(),
    ];
    let mut vectors_dir = PathBuf::from("conformance/vectors");
    let mut fixtures_dir = PathBuf::from("conformance/fixtures");
    let mut output = PathBuf::from("conformance/reports/replay-drift.json");
    let mut update_fixtures = false;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                let value = args.next().ok_or_else(|| anyhow!("missing value for --repo-root"))?;
                repo_root = PathBuf::from(value);
            }
            "--profiles" => {
                let value = args.next().ok_or_else(|| anyhow!("missing value for --profiles"))?;
                profiles = value
                    .split(',')
                    .map(str::trim)
                    .filter(|item| !item.is_empty())
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
            }
            "--vectors-dir" => {
                let value = args.next().ok_or_else(|| anyhow!("missing value for --vectors-dir"))?;
                vectors_dir = PathBuf::from(value);
            }
            "--fixtures-dir" => {
                let value =
                    args.next().ok_or_else(|| anyhow!("missing value for --fixtures-dir"))?;
                fixtures_dir = PathBuf::from(value);
            }
            "--output" => {
                let value = args.next().ok_or_else(|| anyhow!("missing value for --output"))?;
                output = PathBuf::from(value);
            }
            "--update-fixtures" => {
                update_fixtures = true;
            }
            "--help" | "-h" => {
                println!(
                    "usage: replay-drift-validator [--repo-root <path>] [--profiles <csv>] [--vectors-dir <path>] [--fixtures-dir <path>] [--output <path>] [--update-fixtures]"
                );
                std::process::exit(0);
            }
            unknown => {
                bail!("unknown argument {}", unknown);
            }
        }
    }

    Ok(Args {
        repo_root,
        profiles,
        vectors_dir,
        fixtures_dir,
        output,
        update_fixtures,
    })
}

fn profile_vector_name(profile: &str) -> Result<&'static str> {
    match profile {
        "uas-small" => Ok("kernel-smoke.json"),
        "fixed-wing" => Ok("kernel-smoke-fixed-wing.json"),
        "hybrid-vtol" => Ok("kernel-smoke-hybrid-vtol.json"),
        _ => bail!("unsupported profile {}", profile),
    }
}

fn lcg_step(state: u64) -> u64 {
    state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407)
}

fn signed_noise(seed: u64, tick_idx: u64, channel: u64, amplitude: f64) -> f64 {
    let mut state = seed
        ^ (tick_idx.wrapping_add(1)).wrapping_mul(0x9E3779B97F4A7C15)
        ^ channel.wrapping_mul(0xBF58476D1CE4E5B9);
    state = lcg_step(state);
    let unit = ((state >> 11) as f64) / ((1u64 << 53) as f64);
    (unit - 0.5) * 2.0 * amplitude
}

fn run_sequence(
    repo_root: &Path,
    profile: &str,
    base_input: &KernelInput,
    seed: u64,
    ticks: u64,
    time_step_ms: u64,
) -> Result<SimResult> {
    let mut runtime = Runtime::from_repo(repo_root, profile)
        .with_context(|| format!("failed to initialize runtime for {}", profile))?;
    let mut last_verdict = None;

    for idx in 0..ticks {
        let mut input = base_input.clone();
        input.tick.seq = idx + 1;
        input.tick.ts_ms = idx.saturating_mul(time_step_ms);

        for axis in 0..3 {
            let delta = signed_noise(seed, idx, axis as u64, 0.15);
            input.intent.desired_rates_dps[axis] += delta;
        }
        input.intent.desired_climb_mps += signed_noise(seed, idx, 10, 0.1);
        input.state.bank_deg += signed_noise(seed, idx, 11, 2.0);
        input.state.bank_deg = input.state.bank_deg.clamp(-45.0, 45.0);
        input.state.input_age_ms = input
            .state
            .input_age_ms
            .saturating_add((idx % 3).min(2));

        let output = runtime.evaluate(&input);
        last_verdict = Some(format!("{:?}", output.verdict));
    }

    let final_verdict = last_verdict.ok_or_else(|| anyhow!("no ticks evaluated"))?;
    Ok(SimResult {
        final_verdict,
        tip_hash: runtime.tip_hash(),
    })
}

fn validate_profile(
    args: &Args,
    repo_root: &Path,
    profile: &str,
) -> Result<(ProfileReplayReport, Option<ReplayFixture>)> {
    let vector_file = profile_vector_name(profile)?;
    let vector_path = repo_root.join(&args.vectors_dir).join(vector_file);
    let fixture_path = repo_root
        .join(&args.fixtures_dir)
        .join(format!("replay-seed-{}.json", profile));

    let vector: KernelVector = serde_json::from_str(
        &fs::read_to_string(&vector_path)
            .with_context(|| format!("failed reading {}", vector_path.display()))?,
    )
    .with_context(|| format!("invalid JSON {}", vector_path.display()))?;

    let mut fixture: ReplayFixture = serde_json::from_str(
        &fs::read_to_string(&fixture_path)
            .with_context(|| format!("failed reading {}", fixture_path.display()))?,
    )
    .with_context(|| format!("invalid JSON {}", fixture_path.display()))?;

    if vector.profile != profile {
        bail!(
            "vector profile mismatch for {}: vector={}, expected={}",
            profile,
            vector.profile,
            profile
        );
    }
    if fixture.profile != profile {
        bail!(
            "fixture profile mismatch for {}: fixture={}, expected={}",
            profile,
            fixture.profile,
            profile
        );
    }
    if fixture.ticks == 0 {
        bail!("fixture ticks must be > 0 for {}", profile);
    }
    if fixture.time_step_ms == 0 {
        bail!("fixture time_step_ms must be > 0 for {}", profile);
    }

    let baseline_a = run_sequence(
        repo_root,
        profile,
        &vector.input,
        fixture.seed,
        fixture.ticks,
        fixture.time_step_ms,
    )?;
    let baseline_b = run_sequence(
        repo_root,
        profile,
        &vector.input,
        fixture.seed,
        fixture.ticks,
        fixture.time_step_ms,
    )?;
    let drift_seed = fixture.seed.saturating_add(fixture.drift.seed_offset);
    let drift = run_sequence(
        repo_root,
        profile,
        &vector.input,
        drift_seed,
        fixture.ticks,
        fixture.time_step_ms,
    )?;

    if args.update_fixtures {
        fixture.expected_final_verdict = baseline_a.final_verdict.clone();
        fixture.expected_tip_hash = baseline_a.tip_hash.clone();
    }

    let mut failures = Vec::new();
    if baseline_a.final_verdict != baseline_b.final_verdict {
        failures.push("same-seed final verdict mismatch across runs".to_string());
    }
    if baseline_a.tip_hash != baseline_b.tip_hash {
        failures.push("same-seed tip hash mismatch across runs".to_string());
    }
    if !fixture.expected_final_verdict.is_empty()
        && baseline_a.final_verdict != fixture.expected_final_verdict
    {
        failures.push(format!(
            "baseline verdict {} != fixture expected {}",
            baseline_a.final_verdict, fixture.expected_final_verdict
        ));
    }
    if !fixture.expected_tip_hash.is_empty() && baseline_a.tip_hash != fixture.expected_tip_hash {
        failures.push(format!(
            "baseline tip hash {} != fixture expected {}",
            baseline_a.tip_hash, fixture.expected_tip_hash
        ));
    }
    if fixture.drift.require_tip_hash_change && baseline_a.tip_hash == drift.tip_hash {
        failures.push("drift seed did not change tip hash".to_string());
    }
    if fixture.drift.require_verdict_stable && baseline_a.final_verdict != drift.final_verdict {
        failures.push(format!(
            "drift changed verdict {} -> {} despite require_verdict_stable=true",
            baseline_a.final_verdict, drift.final_verdict
        ));
    }

    let result = if failures.is_empty() { "PASS" } else { "FAIL" }.to_string();
    let report = ProfileReplayReport {
        profile: profile.to_string(),
        fixture_id: fixture.fixture_id.clone(),
        baseline_tip_hash: baseline_a.tip_hash,
        baseline_final_verdict: baseline_a.final_verdict,
        repeated_tip_hash: baseline_b.tip_hash,
        repeated_final_verdict: baseline_b.final_verdict,
        drift_tip_hash: drift.tip_hash,
        drift_final_verdict: drift.final_verdict,
        expected_tip_hash: fixture.expected_tip_hash.clone(),
        expected_final_verdict: fixture.expected_final_verdict.clone(),
        result,
        failures,
    };

    let maybe_updated = if args.update_fixtures {
        Some(fixture)
    } else {
        None
    };
    Ok((report, maybe_updated))
}

fn main() -> Result<()> {
    let args = parse_args()?;
    let repo_root = args
        .repo_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", args.repo_root.display()))?;

    let mut profile_reports = Vec::new();
    let mut updates = Vec::new();
    for profile in &args.profiles {
        let (report, maybe_update) = validate_profile(&args, &repo_root, profile)?;
        if let Some(updated) = maybe_update {
            updates.push((profile.clone(), updated));
        }
        println!(
            "{}: {} (baseline verdict {}, tip {})",
            report.profile, report.result, report.baseline_final_verdict, report.baseline_tip_hash
        );
        profile_reports.push(report);
    }

    if args.update_fixtures {
        for (profile, fixture) in updates {
            let path = repo_root
                .join(&args.fixtures_dir)
                .join(format!("replay-seed-{}.json", profile));
            fs::write(&path, serde_json::to_string_pretty(&fixture)? + "\n")
                .with_context(|| format!("failed writing {}", path.display()))?;
        }
    }

    let overall_fail = profile_reports.iter().any(|report| report.result == "FAIL");
    let aggregate = ReplayDriftReport {
        validator: "asc-conformance-kernel/replay-drift-validator@0.1.0".to_string(),
        profiles: profile_reports,
        result: if overall_fail { "FAIL" } else { "PASS" }.to_string(),
    };

    let output_path = repo_root.join(&args.output);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed creating {}", parent.display()))?;
    }
    fs::write(&output_path, serde_json::to_string_pretty(&aggregate)? + "\n")
        .with_context(|| format!("failed writing {}", output_path.display()))?;
    println!("report: {}", output_path.display());

    if overall_fail {
        bail!("replay drift validation failed");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_args_from, profile_vector_name, signed_noise};
    use std::path::PathBuf;

    #[test]
    fn profile_vector_mapping_covers_all_supported_profiles() {
        assert_eq!(
            profile_vector_name("uas-small").expect("uas-small mapping"),
            "kernel-smoke.json"
        );
        assert_eq!(
            profile_vector_name("fixed-wing").expect("fixed-wing mapping"),
            "kernel-smoke-fixed-wing.json"
        );
        assert_eq!(
            profile_vector_name("hybrid-vtol").expect("hybrid-vtol mapping"),
            "kernel-smoke-hybrid-vtol.json"
        );
        assert!(profile_vector_name("unsupported-profile").is_err());
    }

    #[test]
    fn signed_noise_is_deterministic_for_same_inputs() {
        let left = signed_noise(42, 7, 3, 0.15);
        let right = signed_noise(42, 7, 3, 0.15);
        assert_eq!(left, right);
    }

    #[test]
    fn signed_noise_varies_for_different_inputs() {
        let baseline = signed_noise(42, 7, 3, 0.15);
        let different_seed = signed_noise(43, 7, 3, 0.15);
        assert_ne!(baseline, different_seed);
    }

    #[test]
    fn parse_args_from_applies_overrides() {
        let args = vec![
            "--repo-root".to_string(),
            "/tmp/tasc".to_string(),
            "--profiles".to_string(),
            "fixed-wing,hybrid-vtol".to_string(),
            "--output".to_string(),
            "out/report.json".to_string(),
            "--update-fixtures".to_string(),
        ];
        let parsed = parse_args_from(args.into_iter()).expect("parse args");
        assert_eq!(parsed.repo_root, PathBuf::from("/tmp/tasc"));
        assert_eq!(
            parsed.profiles,
            vec!["fixed-wing".to_string(), "hybrid-vtol".to_string()]
        );
        assert_eq!(parsed.output, PathBuf::from("out/report.json"));
        assert!(parsed.update_fixtures);
    }

    #[test]
    fn parse_args_from_rejects_unknown_flags() {
        let args = vec!["--unknown".to_string()];
        assert!(parse_args_from(args.into_iter()).is_err());
    }
}
