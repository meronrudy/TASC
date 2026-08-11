use tasc_sdk::constructors::{minimal_local, minimal_replay, minimal_signed};

#[test]
fn minimal_local_picks_demo_profile() {
    let request = minimal_local();
    assert!(request.target_path.ends_with("examples/minimal-local"));
    assert_eq!(request.trust_mode.as_deref(), Some("dev-local"));
    assert_eq!(request.profile_bundle.as_deref(), Some("dev-local@1.0.0"));
    assert_eq!(request.profile.as_deref(), Some("uas-small"));
}

#[test]
fn minimal_signed_picks_signed_example() {
    let request = minimal_signed();
    assert!(request.target_path.ends_with("examples/minimal-signed"));
    assert_eq!(request.trust_mode.as_deref(), Some("staging-signed"));
    assert_eq!(
        request.profile_bundle.as_deref(),
        Some("staging-signed@1.0.0")
    );
    assert_eq!(request.profile.as_deref(), Some("fixed-wing"));
}

#[test]
fn minimal_replay_picks_replay_example() {
    let request = minimal_replay();
    assert!(request.target_path.ends_with("examples/minimal-replay"));
    assert_eq!(request.trust_mode.as_deref(), Some("dev-local"));
    assert_eq!(request.profile_bundle.as_deref(), Some("dev-local@1.0.0"));
    assert_eq!(request.profile.as_deref(), Some("hybrid-vtol"));
}
