use anyhow::{anyhow, bail, Context, Result};
use asc_kernel_runtime::Runtime;
use asc_reference_adapter::{normalize_to_kernel_intent, AdapterEnvelope};
use asc_reference_interlock::{InterlockDecision, InterlockGate};
use asc_reference_supervisor::{build_incident_initial, build_incident_pack, EvidenceRefs};
use asc_types::model::{Intent, KernelInput, ObservedState, Tick};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Deserialize)]
struct MissionInput {
    #[serde(rename = "missionId")]
    mission_id: String,
    profile: String,
    #[serde(default = "default_ticks")]
    ticks: u64,
    #[serde(default = "default_step_ms", rename = "timeStepMs")]
    time_step_ms: u64,
    #[serde(default = "default_command", rename = "commandVector")]
    command_vector: Vec<f64>,
    #[serde(default = "default_authority", rename = "authoritySource")]
    authority_source: String,
    #[serde(default = "default_heartbeat_hz", rename = "heartbeatHz")]
    heartbeat_hz: f64,
    #[serde(default = "default_true", rename = "interlockEnabled")]
    interlock_enabled: bool,
    #[serde(default = "default_true", rename = "withinSafetyEnvelope")]
    within_safety_envelope: bool,
    #[serde(default = "default_state", rename = "initialState")]
    initial_state: StateInput,
}

#[derive(Debug, Clone, Deserialize)]
struct StateInput {
    #[serde(default = "default_frame")]
    frame: String,
    #[serde(default = "default_position", rename = "positionM")]
    position_m: [f64; 3],
    #[serde(default = "default_velocity", rename = "velocityMps")]
    velocity_mps: f64,
    #[serde(default = "default_bank", rename = "bankDeg")]
    bank_deg: f64,
    #[serde(default = "default_soc", rename = "socPercent")]
    soc_percent: f64,
}

#[derive(Debug, Clone, Serialize)]
struct TraceTick {
    tick: u64,
    ts_ms: u64,
    kernel_input: KernelInput,
    expected_verdict: String,
    expected_reasons: Vec<String>,
    expected_tip_hash: String,
}

fn default_ticks() -> u64 {
    100
}
fn default_step_ms() -> u64 {
    20
}
fn default_command() -> Vec<f64> {
    vec![0.25, 0.0, 0.0, 0.0]
}
fn default_authority() -> String {
    "safety_kernel".to_string()
}
fn default_heartbeat_hz() -> f64 {
    12.0
}
fn default_true() -> bool {
    true
}
fn default_frame() -> String {
    "NED".to_string()
}
fn default_position() -> [f64; 3] {
    [0.0, 0.0, 25.0]
}
fn default_velocity() -> f64 {
    10.0
}
fn default_bank() -> f64 {
    0.0
}
fn default_soc() -> f64 {
    90.0
}
fn default_state() -> StateInput {
    StateInput {
        frame: default_frame(),
        position_m: default_position(),
        velocity_mps: default_velocity(),
        bank_deg: default_bank(),
        soc_percent: default_soc(),
    }
}

fn utc_now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs as i64, 0)
        .unwrap()
        .to_rfc3339()
        .replace("+00:00", "Z")
}

fn sha_prefixed(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("sha256:{:x}", hasher.finalize())
}

fn digest_json(value: &serde_json::Value) -> String {
    let payload = serde_json::to_vec(value).unwrap_or_default();
    sha_prefixed(&payload)
}

fn build_kernel_input(mission: &MissionInput, tick: u64, ts_ms: u64, allowed: bool) -> KernelInput {
    let mut rates = [0.0, 0.0, 0.0];
    for (idx, rate) in rates.iter_mut().enumerate() {
        let source = mission.command_vector.get(idx).copied().unwrap_or(0.0);
        *rate = if allowed { source } else { 0.0 };
    }
    let climb = if allowed {
        mission.command_vector.get(3).copied().unwrap_or(0.0)
    } else {
        0.0
    };
    KernelInput {
        tick: Tick { seq: tick, ts_ms },
        state: ObservedState {
            frame: mission.initial_state.frame.clone(),
            position_m: mission.initial_state.position_m,
            velocity_mps: mission.initial_state.velocity_mps,
            bank_deg: mission.initial_state.bank_deg,
            soc_percent: mission.initial_state.soc_percent,
            input_age_ms: 0,
        },
        intent: Intent {
            desired_rates_dps: rates,
            desired_climb_mps: climb,
        },
    }
}

fn render_signed_log(
    runtime: &Runtime,
    mission_id: &str,
    profile: &str,
    timestamp_utc: &str,
) -> serde_json::Value {
    let mut events = Vec::new();
    let mut prev = String::new();
    for record in &runtime.log.records {
        let payload = serde_json::to_string(&record.payload).unwrap_or_else(|_| "{}".to_string());
        let payload_digest = sha_prefixed(payload.as_bytes());
        let event_material = format!(
            "{}|{}|{}|kernel_tick|{}",
            record.seq,
            prev,
            payload_digest,
            record.seq.saturating_mul(20)
        );
        let event_hash = sha_prefixed(event_material.as_bytes());
        events.push(json!({
            "seq": record.seq,
            "tickMs": record.seq.saturating_mul(20),
            "eventType": "kernel_tick",
            "payloadDigest": payload_digest,
            "prevHash": prev,
            "hash": event_hash
        }));
        prev = event_hash;
    }
    let signature = base64::engine::general_purpose::STANDARD.encode(sha_prefixed(prev.as_bytes()));
    json!({
        "schemaVersion": "0.2",
        "missionId": mission_id,
        "profile": profile,
        "algorithm": "sha256-chain-v1",
        "root": prev,
        "signature": signature,
        "signatureAlgorithm": "sha256-digest",
        "signingTimeUtc": timestamp_utc,
        "events": events
    })
}

fn parse_args() -> Result<(PathBuf, String, PathBuf, PathBuf)> {
    let mut args = env::args().skip(1);
    let cmd = args.next().ok_or_else(|| anyhow!("missing command"))?;
    if cmd != "run-mission" {
        bail!("unsupported command {}", cmd);
    }
    let mut repo_root = PathBuf::from(".");
    let mut profile = String::new();
    let mut mission_input = PathBuf::new();
    let mut out_dir = PathBuf::new();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--repo-root" => {
                repo_root = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?)
            }
            "--profile" => profile = args.next().ok_or_else(|| anyhow!("missing value"))?,
            "--mission-input" => {
                mission_input = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?)
            }
            "--out-dir" => {
                out_dir = PathBuf::from(args.next().ok_or_else(|| anyhow!("missing value"))?)
            }
            _ => bail!("unknown argument {}", arg),
        }
    }
    if profile.is_empty() || mission_input.as_os_str().is_empty() || out_dir.as_os_str().is_empty()
    {
        bail!("usage: run-mission --profile <p> --mission-input <json> --out-dir <path> [--repo-root <path>]");
    }
    Ok((repo_root, profile, mission_input, out_dir))
}

fn main() -> Result<()> {
    let (repo_root, profile, mission_input_path, out_dir) = parse_args()?;
    let repo_root = repo_root.canonicalize().unwrap_or(repo_root);
    let out_dir = out_dir
        .canonicalize()
        .unwrap_or_else(|_| repo_root.join(out_dir));
    fs::create_dir_all(&out_dir)?;

    let mission: MissionInput = serde_json::from_str(
        &fs::read_to_string(&mission_input_path)
            .with_context(|| format!("failed reading {}", mission_input_path.display()))?,
    )?;
    if mission.profile != profile {
        bail!(
            "mission profile mismatch: input {}, arg {}",
            mission.profile,
            profile
        );
    }

    let mut runtime = Runtime::from_repo(&repo_root, &profile)?;
    let mut gate = InterlockGate::new(10.0);
    let mut trace_ticks: Vec<TraceTick> = Vec::new();
    let mut latest_decision = InterlockDecision {
        allowed: true,
        reason: "authorized".to_string(),
        latched: false,
    };
    let timestamp_utc = utc_now_iso();

    for tick in 1..=mission.ticks {
        let envelope = AdapterEnvelope {
            profile: profile.clone(),
            mission_id: mission.mission_id.clone(),
            mode_request: "AUTO".to_string(),
            command_vector: mission.command_vector.clone(),
            authority_source: mission.authority_source.clone(),
        };
        let normalized = normalize_to_kernel_intent(&envelope, timestamp_utc.clone())?;
        latest_decision = gate.evaluate(
            &normalized.authority_source,
            mission.heartbeat_hz,
            mission.within_safety_envelope,
            mission.interlock_enabled,
        );
        let ts_ms = tick.saturating_mul(mission.time_step_ms);
        let kernel_input = build_kernel_input(&mission, tick, ts_ms, latest_decision.allowed);
        let kernel_output = runtime.evaluate(&kernel_input);
        trace_ticks.push(TraceTick {
            tick,
            ts_ms,
            kernel_input,
            expected_verdict: format!("{:?}", kernel_output.verdict),
            expected_reasons: kernel_output
                .reasons
                .iter()
                .map(|reason| format!("{:?}", reason))
                .collect::<Vec<_>>(),
            expected_tip_hash: runtime.tip_hash(),
        });
    }

    let signed_log = render_signed_log(&runtime, &mission.mission_id, &profile, &timestamp_utc);
    let signed_log_path = out_dir.join(format!(
        "signed-operational-log-{}.json",
        mission.mission_id
    ));
    fs::write(
        &signed_log_path,
        serde_json::to_string_pretty(&signed_log)? + "\n",
    )?;

    let refs = EvidenceRefs {
        evidence_map_hash: sha_prefixed(profile.as_bytes()),
        signed_log_segment_hash: digest_json(&signed_log),
        replay_recipe_hash: sha_prefixed(format!("replay:{}", mission.mission_id).as_bytes()),
        attestation_hash: sha_prefixed(format!("attestation:{}", mission.mission_id).as_bytes()),
    };
    let severity = if latest_decision.allowed { "S0" } else { "S2" };
    let initial = build_incident_initial(
        &format!("sys:{}:{}", profile, mission.mission_id),
        &profile,
        severity,
        if latest_decision.allowed {
            "Mission completed without safety intervention."
        } else {
            "Interlock denied authority during mission."
        },
        &refs,
        &latest_decision,
        &timestamp_utc,
    );
    let initial_path = out_dir.join(format!("incident-initial-{}.json", mission.mission_id));
    fs::write(
        &initial_path,
        serde_json::to_string_pretty(&initial)? + "\n",
    )?;

    let pack = build_incident_pack(
        &initial,
        &sha_prefixed(b"corrective-action-plan"),
        &timestamp_utc,
    );
    let pack_path = out_dir.join(format!("incident-pack-{}.json", mission.mission_id));
    fs::write(&pack_path, serde_json::to_string_pretty(&pack)? + "\n")?;

    let summary = json!({
        "missionId": mission.mission_id,
        "profile": profile,
        "ticks": mission.ticks,
        "timeStepMs": mission.time_step_ms,
        "finalTipHash": runtime.tip_hash(),
        "finalVerdict": trace_ticks.last().map(|item| item.expected_verdict.clone()).unwrap_or_else(|| "Allow".to_string()),
        "interlock": latest_decision,
        "artifacts": {
            "signedOperationalLog": signed_log_path.file_name().and_then(|f| f.to_str()).unwrap_or_default(),
            "incidentInitial": initial_path.file_name().and_then(|f| f.to_str()).unwrap_or_default(),
            "incidentPack": pack_path.file_name().and_then(|f| f.to_str()).unwrap_or_default()
        }
    });
    let summary_path = out_dir.join(format!("mission-summary-{}.json", mission.mission_id));
    fs::write(
        &summary_path,
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;

    let trace = json!({
        "schemaVersion": "0.1",
        "missionId": mission.mission_id,
        "profile": profile,
        "tickCount": trace_ticks.len(),
        "ticks": trace_ticks,
        "finalTipHash": runtime.tip_hash(),
    });
    let trace_path = out_dir.join(format!("operational-trace-{}.json", mission.mission_id));
    fs::write(&trace_path, serde_json::to_string_pretty(&trace)? + "\n")?;

    println!("wrote {}", signed_log_path.display());
    println!("wrote {}", initial_path.display());
    println!("wrote {}", pack_path.display());
    println!("wrote {}", summary_path.display());
    println!("wrote {}", trace_path.display());
    Ok(())
}
