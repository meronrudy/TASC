use asc_kernel_runtime::Runtime;
use asc_types::{
    model::{Intent, KernelInput, ObservedState, Tick},
    ReasonCode, Verdict,
};

fn baseline_input(seq: u64, ts_ms: u64) -> KernelInput {
    KernelInput {
        tick: Tick { seq, ts_ms },
        state: ObservedState {
            frame: "NED".into(),
            position_m: [0.0, 0.0, 20.0],
            velocity_mps: 0.0,
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
fn invariant_violation_forces_shutdown() {
    let mut runtime = Runtime::new("fingerprint".into());
    let mut input = baseline_input(1, 0);
    input.state.position_m[2] = 2.0;
    input.state.bank_deg = 70.0;

    let out = runtime.evaluate(&input);
    assert_eq!(out.verdict, Verdict::Shutdown);
    assert!(out.command.shutdown);
    assert!(out.reasons.contains(&ReasonCode::InvariantViolation));
}

#[test]
fn shutdown_precedence_beats_override() {
    let mut runtime = Runtime::new("fingerprint".into());
    let _ = runtime.evaluate(&baseline_input(1, 0));

    let mut input = baseline_input(2, 25);
    input.state.position_m[2] = 0.0;

    let out = runtime.evaluate(&input);
    assert!(out.reasons.contains(&ReasonCode::DeadlineMiss));
    assert!(out.reasons.contains(&ReasonCode::InvariantViolation));
    assert_eq!(out.verdict, Verdict::Shutdown);
}

#[test]
fn non_finite_state_value_forces_shutdown() {
    let mut runtime = Runtime::new("fingerprint".into());
    let mut input = baseline_input(1, 0);
    input.state.velocity_mps = f64::NAN;

    let out = runtime.evaluate(&input);
    assert_eq!(out.verdict, Verdict::Shutdown);
    assert!(out.reasons.contains(&ReasonCode::StateOutOfBounds));
}

#[test]
fn position_out_of_bounds_forces_clamp() {
    let mut runtime = Runtime::new("fingerprint".into());
    let mut input = baseline_input(1, 0);
    input.state.position_m = [6000.0, 0.0, 20.0];

    let out = runtime.evaluate(&input);
    assert_eq!(out.verdict, Verdict::Clamp);
    assert!(out.reasons.contains(&ReasonCode::StateOutOfBounds));
}
