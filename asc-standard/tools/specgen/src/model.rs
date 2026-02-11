#![allow(dead_code)]

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TupleSpec {
    pub version: String,
    pub reason_codes: Vec<String>,
    pub severities: Vec<String>,
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PositionBounds {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuaranteesSpec {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvariantsSpec {
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileSpec {
    pub name: String,
    pub timing: ProfileTiming,
    pub capabilities: ProfileCapabilities,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileTiming {
    pub control_hz: u64,
    pub deadline_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileCapabilities {
    pub vtol: bool,
    pub fixed_wing: bool,
    pub max_payload_kg: f64,
}
