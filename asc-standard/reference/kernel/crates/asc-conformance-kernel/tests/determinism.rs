use asc_kernel_runtime::Runtime;
use asc_types::{
    model::{Intent, KernelInput, ObservedState, Tick},
    ReasonCode, Verdict,
};

fn sample_input(seq: u64, ts_ms: u64) -> KernelInput {
    KernelInput {
        tick: Tick { seq, ts_ms },
        state: ObservedState {
            frame: "NED".to_string(),
            position_m: [10.0, 0.0, 20.0],
            velocity_mps: 12.0,
            bank_deg: 5.0,
            soc_percent: 80.0,
            input_age_ms: 10,
        },
        intent: Intent {
            desired_rates_dps: [1.0, 2.0, 3.0],
            desired_climb_mps: 1.5,
        },
    }
}

#[test]
fn replay_tip_hash_is_deterministic() {
    let mut run_a = Runtime::new("fingerprint".into());
    let mut run_b = Runtime::new("fingerprint".into());

    for idx in 0..25 {
        let input = sample_input(idx, idx * 20);
        let _ = run_a.evaluate(&input);
        let _ = run_b.evaluate(&input);
    }

    assert_eq!(run_a.tip_hash(), run_b.tip_hash());
}

#[test]
fn temporal_violation_and_deadline_miss_are_deterministic() {
    let mut runtime = Runtime::new("fingerprint".into());
    let _ = runtime.evaluate(&sample_input(1, 0));

    let out = runtime.evaluate(&sample_input(2, 150));
    assert_eq!(out.verdict, Verdict::Override);
    assert!(out
        .reasons
        .contains(&ReasonCode::TemporalGuaranteeViolation));
    assert!(out.reasons.contains(&ReasonCode::DeadlineMiss));
}

#[test]
fn attitude_violation_is_deterministic() {
    let mut run_a = Runtime::new("fingerprint".into());
    let mut run_b = Runtime::new("fingerprint".into());
    let mut input = sample_input(1, 0);
    input.state.bank_deg = 85.0;

    let out_a = run_a.evaluate(&input);
    let out_b = run_b.evaluate(&input);
    assert_eq!(out_a.verdict, Verdict::Shutdown);
    assert_eq!(out_a.verdict, out_b.verdict);
    assert_eq!(out_a.reasons, out_b.reasons);
    assert!(out_a.reasons.contains(&ReasonCode::InvariantViolation));
}
