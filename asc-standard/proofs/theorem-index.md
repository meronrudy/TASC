# Theorem Index

Proof obligations map to requirements, executable tests, and immutable evidence artifacts.

| Theorem ID | Claim | Requirement Link | Spec Link | Test Link | Evidence Link |
| --- | --- | --- | --- | --- | --- |
| THM-INV-001 | Invariant violations force deterministic shutdown behavior. | `REQ-002` | `spec/asc/invariants-rcbf.yaml#min_altitude_m` | `TST-INV-001` | `evidence/manifests/kernel-test-uas-small.json` |
| THM-TEMP-001 | Temporal guarantee violations raise override reasons with deterministic precedence. | `REQ-003` | `spec/asc/guarantees-stl.yaml#deadline_ms` | `TST-GUA-001` | `evidence/manifests/temporal-guarantee-uas-small.json` |
| THM-RPY-001 | Replay execution with identical seeds/build/config/environment reproduces verdict and tip hash. | `REQ-004` | `spec/interfaces/bus-mapping.md#determinism-notes` | `TST-RPY-001` | `conformance/reports/replay-drift.json` |
| THM-TASC-TA2-001 | TA2 attestation evidence remains verifier-checkable with trust-chain and revocation checks. | `REQ-006` | `spec/tasc/checks.yaml#CHK_ATTESTATION_TA2` | `TST-TASC-002` | `evidence/manifests/tasc-conformance-uas-small.json` |
| THM-TASC-TP-001 | Dual transparency proofs are required and independently verifiable for release acceptance. | `REQ-006` | `spec/tasc/checks.yaml#CHK_TRANSPARENCY_REKOR_PROOF` | `TST-TASC-003` | `evidence/manifests/tasc-conformance-fixed-wing.json` |
| THM-TASC-DATA-001 | Incident/logging/replay taxonomies stay schema-compatible and version-impact declared. | `REQ-007` | `data/version-impact-map.yaml` | `TST-TASC-004` | `conformance/reports/data-version-impact.json` |
| THM-TASC-GOV-001 | Governance authority/escalation controls are release-gated and branch protection is enforced. | `REQ-008` | `governance/COMPLIANCE_POLICY.md` | `TST-TASC-005` | `conformance/reports/governance-policy-gate.json` |

## Traceability Notes

- Requirement IDs resolve via `safety-case/traceability/req_to_spec.csv`.
- Test IDs resolve via `safety-case/traceability/spec_to_test.csv`.
- Evidence artifacts resolve via `safety-case/traceability/test_to_evidence.csv` and release manifests.
- Assumption dependencies are indexed in `proofs/assumptions-register.md` and should be updated when theorem links change.
