use anyhow::Result;
use asc_contract::{load_contract, ContractBundle};
use asc_kernel_model::{constrain, KernelLimits};
use asc_logging::EventLog;
use asc_types::model::{KernelInput, KernelOutput};
use std::path::Path;

pub struct Runtime {
    contract_fingerprint: String,
    limits: KernelLimits,
    last_tick_ts_ms: Option<u64>,
    pub log: EventLog,
}

impl Runtime {
    pub fn new(contract_fingerprint: String) -> Self {
        Self::with_limits(contract_fingerprint, KernelLimits::default())
    }

    pub fn with_limits(contract_fingerprint: String, limits: KernelLimits) -> Self {
        Self {
            contract_fingerprint,
            limits,
            last_tick_ts_ms: None,
            log: EventLog::default(),
        }
    }

    pub fn from_repo(repo_root: &Path, profile_name: &str) -> Result<Self> {
        let bundle = load_contract(repo_root, profile_name)?;
        let limits = limits_from_contract(&bundle);
        Ok(Self::with_limits(bundle.fingerprint, limits))
    }

    pub fn evaluate(&mut self, input: &KernelInput) -> KernelOutput {
        let inter_tick_ms = self
            .last_tick_ts_ms
            .map(|prev| input.tick.ts_ms.saturating_sub(prev));

        let mut output = constrain(input, inter_tick_ms, &self.limits);
        output.contract_fingerprint = self.contract_fingerprint.clone();
        self.log.append(input.tick.seq, &output);
        self.last_tick_ts_ms = Some(input.tick.ts_ms);
        output
    }

    pub fn tip_hash(&self) -> String {
        self.log.tip_hash.clone()
    }
}

fn limits_from_contract(bundle: &ContractBundle) -> KernelLimits {
    KernelLimits {
        frame: bundle.state.frame.clone(),
        max_speed_mps: bundle.state.max_speed_mps,
        max_bank_deg: bundle.invariants.max_bank_deg,
        attitude_limit_deg: bundle.state.attitude_limit_deg,
        min_altitude_m: bundle.invariants.min_altitude_m,
        position_min_m: bundle.state.position_bounds_m.min,
        position_max_m: bundle.state.position_bounds_m.max,
        min_soc_percent: bundle.energy.min_soc_percent,
        max_input_age_ms: bundle.guarantees.max_input_age_ms,
        max_tick_interval_ms: bundle.guarantees.max_tick_interval_ms,
        deadline_ms: bundle.guarantees.deadline_ms,
        max_roll_rate_dps: bundle.flow.max_roll_rate_dps,
        max_pitch_rate_dps: bundle.flow.max_pitch_rate_dps,
        max_yaw_rate_dps: bundle.flow.max_yaw_rate_dps,
        max_climb_rate_mps: bundle.flow.max_climb_rate_mps,
    }
}

#[cfg(test)]
mod tests {
    use super::Runtime;
    use asc_kernel_model::KernelLimits;
    use asc_types::{
        model::{Intent, KernelInput, ObservedState, Tick},
        ReasonCode, Verdict,
    };

    fn baseline_input() -> KernelInput {
        KernelInput {
            tick: Tick { seq: 1, ts_ms: 0 },
            state: ObservedState {
                frame: "NED".into(),
                position_m: [0.0, 0.0, 20.0],
                velocity_mps: 10.0,
                bank_deg: 0.0,
                soc_percent: 90.0,
                input_age_ms: 0,
            },
            intent: Intent {
                desired_rates_dps: [0.0, 0.0, 0.0],
                desired_climb_mps: 0.0,
            },
        }
    }

    #[test]
    fn with_limits_enforces_custom_thresholds() {
        let mut limits = KernelLimits::default();
        limits.max_speed_mps = 5.0;
        let mut runtime = Runtime::with_limits("fp".into(), limits);
        let input = baseline_input();

        let out = runtime.evaluate(&input);
        assert_eq!(out.verdict, Verdict::Clamp);
        assert!(out.reasons.contains(&ReasonCode::StateOutOfBounds));
    }
}
