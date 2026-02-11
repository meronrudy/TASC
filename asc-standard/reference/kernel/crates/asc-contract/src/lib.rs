use anyhow::{Context, Result};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TupleSpec {
    pub version: String,
    pub reason_codes: Vec<String>,
    pub severities: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateSpec {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub frame: String,
    pub position_bounds_m: PositionBounds,
    pub attitude_limit_deg: f64,
    pub max_speed_mps: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlowSpec {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub max_roll_rate_dps: f64,
    pub max_pitch_rate_dps: f64,
    pub max_yaw_rate_dps: f64,
    pub max_climb_rate_mps: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergySpec {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub min_soc_percent: f64,
    pub reserve_endurance_s: f64,
    pub max_power_w: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Guarantees {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub max_input_age_ms: u64,
    pub max_tick_interval_ms: u64,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invariants {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub min_altitude_m: f64,
    pub max_bank_deg: f64,
    pub require_geofence: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterlockGate {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub intent: String,
    #[serde(default)]
    pub normative_refs: Vec<String>,
    pub armed_required: bool,
    pub fault_latched_shutdown: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub timing: ProfileTiming,
    pub capabilities: ProfileCapabilities,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileTiming {
    pub control_hz: u64,
    pub deadline_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileCapabilities {
    pub vtol: bool,
    pub fixed_wing: bool,
    pub max_payload_kg: f64,
}

#[derive(Debug, Clone)]
pub struct ContractBundle {
    pub tuple: TupleSpec,
    pub state: StateSpec,
    pub flow: FlowSpec,
    pub energy: EnergySpec,
    pub guarantees: Guarantees,
    pub invariants: Invariants,
    pub interlock: InterlockGate,
    pub profile: Profile,
    pub fingerprint: String,
}

pub fn load_contract(repo_root: &Path, profile_name: &str) -> Result<ContractBundle> {
    let tuple_raw = read(repo_root.join("spec/asc/tuple.yaml"))?;
    let state_raw = read(repo_root.join("spec/asc/state-se3.yaml"))?;
    let flow_raw = read(repo_root.join("spec/asc/flow-phs.yaml"))?;
    let energy_raw = read(repo_root.join("spec/asc/energy-contract.yaml"))?;
    let guarantees_raw = read(repo_root.join("spec/asc/guarantees-stl.yaml"))?;
    let inv_raw = read(repo_root.join("spec/asc/invariants-rcbf.yaml"))?;
    let interlock_raw = read(repo_root.join("spec/asc/interlock-gate.yaml"))?;
    let profile_raw = read(repo_root.join(format!("spec/profiles/{profile_name}.yaml")))?;

    let tuple: TupleSpec = serde_yaml::from_str(&tuple_raw)?;
    let state: StateSpec = serde_yaml::from_str(&state_raw)?;
    let flow: FlowSpec = serde_yaml::from_str(&flow_raw)?;
    let energy: EnergySpec = serde_yaml::from_str(&energy_raw)?;
    let guarantees: Guarantees = serde_yaml::from_str(&guarantees_raw)?;
    let invariants: Invariants = serde_yaml::from_str(&inv_raw)?;
    let interlock: InterlockGate = serde_yaml::from_str(&interlock_raw)?;
    let profile: Profile = serde_yaml::from_str(&profile_raw)?;

    let mut hasher = Sha256::new();
    for raw in [
        tuple_raw,
        state_raw,
        flow_raw,
        energy_raw,
        guarantees_raw,
        inv_raw,
        interlock_raw,
        profile_raw,
    ] {
        hasher.update(normalize(&raw));
    }

    Ok(ContractBundle {
        tuple,
        state,
        flow,
        energy,
        guarantees,
        invariants,
        interlock,
        profile,
        fingerprint: hex::encode(hasher.finalize()),
    })
}

fn read(path: impl AsRef<Path>) -> Result<String> {
    let path = path.as_ref();
    fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))
}

fn normalize(input: &str) -> String {
    input
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::load_contract;
    use std::path::PathBuf;

    #[test]
    fn loads_full_contract_bundle() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../")
            .canonicalize()
            .expect("canonical repo root");
        let bundle = load_contract(&repo_root, "uas-small").expect("load contract");
        assert_eq!(bundle.profile.name, "uas-small");
        assert_eq!(bundle.state.frame, "NED");
        assert!(!bundle.fingerprint.is_empty());
    }
}
