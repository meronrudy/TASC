use asc_kernel_runtime::Runtime;
use asc_types::model::{Intent, KernelInput, ObservedState, Tick};
use std::path::PathBuf;

#[test]
fn runtime_from_repo_loads_contract_and_injects_fingerprint() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../")
        .canonicalize()
        .expect("canonical repo root");

    let mut runtime = Runtime::from_repo(&repo_root, "uas-small").expect("runtime from repo");
    let input = KernelInput {
        tick: Tick { seq: 1, ts_ms: 0 },
        state: ObservedState {
            frame: "NED".into(),
            position_m: [0.0, 0.0, 20.0],
            velocity_mps: 10.0,
            bank_deg: 1.0,
            soc_percent: 90.0,
            input_age_ms: 1,
        },
        intent: Intent {
            desired_rates_dps: [0.5, 0.5, 0.5],
            desired_climb_mps: 0.5,
        },
    };

    let out = runtime.evaluate(&input);
    assert!(!out.contract_fingerprint.is_empty());
    assert_eq!(out.contract_fingerprint.len(), 64);
}
