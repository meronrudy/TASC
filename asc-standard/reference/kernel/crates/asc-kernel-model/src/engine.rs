use asc_types::model::{ConstrainedCommand, KernelInput, KernelOutput};
use asc_types::Verdict;

use crate::{arbitration::decide, checks::evaluate_checks, KernelLimits};

pub fn constrain(
    input: &KernelInput,
    inter_tick_ms: Option<u64>,
    limits: &KernelLimits,
) -> KernelOutput {
    let outcomes = evaluate_checks(input, inter_tick_ms, limits);
    let verdict = decide(&outcomes);

    let mut rates = input.intent.desired_rates_dps;
    rates[0] = rates[0].clamp(-limits.max_roll_rate_dps, limits.max_roll_rate_dps);
    rates[1] = rates[1].clamp(-limits.max_pitch_rate_dps, limits.max_pitch_rate_dps);
    rates[2] = rates[2].clamp(-limits.max_yaw_rate_dps, limits.max_yaw_rate_dps);
    let climb = input
        .intent
        .desired_climb_mps
        .clamp(-limits.max_climb_rate_mps, limits.max_climb_rate_mps);

    let command = match verdict {
        Verdict::Allow | Verdict::Clamp => ConstrainedCommand {
            applied_rates_dps: rates,
            applied_climb_mps: climb,
            shutdown: false,
        },
        Verdict::Hold => ConstrainedCommand {
            applied_rates_dps: [0.0, 0.0, 0.0],
            applied_climb_mps: 0.0,
            shutdown: false,
        },
        Verdict::Override => ConstrainedCommand {
            applied_rates_dps: [0.0, 0.0, 0.0],
            applied_climb_mps: -1.0,
            shutdown: false,
        },
        Verdict::Shutdown => ConstrainedCommand {
            applied_rates_dps: [0.0, 0.0, 0.0],
            applied_climb_mps: 0.0,
            shutdown: true,
        },
    };

    KernelOutput {
        verdict,
        reasons: outcomes.iter().map(|o| o.reason).collect(),
        command,
        contract_fingerprint: String::new(),
    }
}
