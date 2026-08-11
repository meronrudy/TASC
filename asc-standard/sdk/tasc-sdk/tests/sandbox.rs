#[cfg(feature = "sandbox")]
mod tests {
    use tasc_sdk::sandbox::{attest, NON_PRODUCTION_MARKER};
    use tasc_sdk::SdkError;

    #[test]
    fn rejects_prod_hsm_trust_mode() {
        let err = attest(Some("prod-hsm")).expect_err("prod-hsm should be rejected");
        match err {
            SdkError::InvalidConfig(message) => assert!(message.contains("prod-hsm")),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn attestation_marks_non_production() {
        let attestation = attest(Some("dev-local")).expect("attestation");
        assert_eq!(attestation.marker, NON_PRODUCTION_MARKER);
        assert_eq!(attestation.trust_mode.as_deref(), Some("dev-local"));
        assert!(attestation.evidence.contains("sandbox"));
    }
}
