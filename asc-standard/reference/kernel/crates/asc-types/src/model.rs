use crate::{ReasonCode, Severity, Verdict};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    pub seq: u64,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservedState {
    pub frame: String,
    pub position_m: [f64; 3],
    pub velocity_mps: f64,
    pub bank_deg: f64,
    pub soc_percent: f64,
    pub input_age_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intent {
    pub desired_rates_dps: [f64; 3],
    pub desired_climb_mps: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstrainedCommand {
    pub applied_rates_dps: [f64; 3],
    pub applied_climb_mps: f64,
    pub shutdown: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckOutcome {
    pub verdict: Verdict,
    pub reason: ReasonCode,
    pub severity: Severity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInput {
    pub tick: Tick,
    pub state: ObservedState,
    pub intent: Intent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelOutput {
    pub verdict: Verdict,
    pub reasons: Vec<ReasonCode>,
    pub command: ConstrainedCommand,
    pub contract_fingerprint: String,
}
