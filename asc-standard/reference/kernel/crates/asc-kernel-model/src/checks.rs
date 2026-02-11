use asc_types::{
    model::{CheckOutcome, KernelInput},
    ReasonCode, Severity, Verdict,
};

use crate::KernelLimits;

pub fn evaluate_checks(
    input: &KernelInput,
    inter_tick_ms: Option<u64>,
    limits: &KernelLimits,
) -> Vec<CheckOutcome> {
    let mut outcomes = Vec::new();

    let state_values = [
        input.state.position_m[0],
        input.state.position_m[1],
        input.state.position_m[2],
        input.state.velocity_mps,
        input.state.bank_deg,
        input.state.soc_percent,
    ];
    if state_values.iter().any(|value| !value.is_finite()) {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Shutdown,
            reason: ReasonCode::StateOutOfBounds,
            severity: Severity::Critical,
        });
    }

    if input.state.frame != limits.frame {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Shutdown,
            reason: ReasonCode::StateInvalidFrame,
            severity: Severity::Critical,
        });
    }
    if input.state.velocity_mps > limits.max_speed_mps {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Clamp,
            reason: ReasonCode::StateOutOfBounds,
            severity: Severity::Warning,
        });
    }
    if input.state.position_m[0] < limits.position_min_m[0]
        || input.state.position_m[0] > limits.position_max_m[0]
        || input.state.position_m[1] < limits.position_min_m[1]
        || input.state.position_m[1] > limits.position_max_m[1]
        || input.state.position_m[2] < limits.position_min_m[2]
        || input.state.position_m[2] > limits.position_max_m[2]
    {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Clamp,
            reason: ReasonCode::StateOutOfBounds,
            severity: Severity::Warning,
        });
    }
    if input.intent.desired_rates_dps[0].abs() > limits.max_roll_rate_dps
        || input.intent.desired_rates_dps[1].abs() > limits.max_pitch_rate_dps
        || input.intent.desired_rates_dps[2].abs() > limits.max_yaw_rate_dps
        || input.intent.desired_climb_mps.abs() > limits.max_climb_rate_mps
    {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Clamp,
            reason: ReasonCode::FlowConstraintViolation,
            severity: Severity::Warning,
        });
    }
    if input.state.soc_percent < limits.min_soc_percent {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Hold,
            reason: ReasonCode::EnergyBudgetExceeded,
            severity: Severity::Critical,
        });
    }
    if input.state.input_age_ms > limits.max_input_age_ms {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Hold,
            reason: ReasonCode::InputStale,
            severity: Severity::Critical,
        });
    }
    if let Some(delta_ms) = inter_tick_ms {
        if delta_ms > limits.max_tick_interval_ms {
            outcomes.push(CheckOutcome {
                verdict: Verdict::Override,
                reason: ReasonCode::TemporalGuaranteeViolation,
                severity: Severity::Critical,
            });
        }
        if delta_ms > limits.deadline_ms {
            outcomes.push(CheckOutcome {
                verdict: Verdict::Override,
                reason: ReasonCode::DeadlineMiss,
                severity: Severity::Critical,
            });
        }
    }
    if input.state.position_m[2] < limits.min_altitude_m
        || input.state.bank_deg.abs() > limits.max_bank_deg
        || input.state.bank_deg.abs() > limits.attitude_limit_deg
    {
        outcomes.push(CheckOutcome {
            verdict: Verdict::Shutdown,
            reason: ReasonCode::InvariantViolation,
            severity: Severity::Critical,
        });
    }

    outcomes
}

#[cfg(test)]
mod tests {
    use super::evaluate_checks;
    use crate::KernelLimits;
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
    fn frame_mismatch_uses_dynamic_limits() {
        let mut input = baseline_input();
        input.state.frame = "ENU".into();

        let limits = KernelLimits::default();
        let outcomes = evaluate_checks(&input, None, &limits);

        assert!(outcomes
            .iter()
            .any(|outcome| outcome.reason == ReasonCode::StateInvalidFrame));
        assert!(outcomes
            .iter()
            .any(|outcome| outcome.verdict == Verdict::Shutdown));
    }

    #[test]
    fn speed_limit_uses_supplied_thresholds() {
        let mut input = baseline_input();
        input.state.velocity_mps = 12.0;

        let mut limits = KernelLimits::default();
        limits.max_speed_mps = 5.0;

        let outcomes = evaluate_checks(&input, None, &limits);
        assert!(outcomes
            .iter()
            .any(|outcome| outcome.reason == ReasonCode::StateOutOfBounds));
        assert!(outcomes
            .iter()
            .any(|outcome| outcome.verdict == Verdict::Clamp));
    }
}
