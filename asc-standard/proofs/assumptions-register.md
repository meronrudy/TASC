# Assumptions Register

| Assumption ID | Assumption | Supports Theorem(s) | Rationale | Validation Hook | Evidence Hook |
| --- | --- | --- | --- | --- | --- |
| A-001 | Sensor timestamps are monotonic within each mission timeline. | `THM-RPY-001`, `THM-TEMP-001` | Required to evaluate inter-tick temporal guarantees deterministically. | `reference/kernel/crates/asc-conformance-kernel/tests/determinism.rs` | `conformance/reports/replay-drift.json` |
| A-002 | Contract fingerprint loaded by runtime identifies the active profile contract unambiguously. | `THM-RPY-001`, `THM-INV-001` | Prevents replay ambiguity across profile boundaries. | `reference/kernel/crates/asc-conformance-kernel/tests/runtime_from_repo.rs` | `evidence/manifests/kernel-test-uas-small.json` |
| A-003 | TA2 signer chain and revocation snapshot are available for offline verification. | `THM-TASC-TA2-001` | Required for non-networked audit and claims workflows. | `tools/tasc-verify/src/main.rs#CHK_ATTESTATION_TA2` | `evidence/manifests/tasc-conformance-hybrid-vtol.json` |
| A-004 | Rekor and mirror checkpoint roots used in proofs are trusted and policy-pinned. | `THM-TASC-TP-001` | Required to detect proof equivocation and stale transparency state. | `tools/tasc-verify/src/main.rs#CHK_TRANSPARENCY_REKOR_PROOF` | `evidence/manifests/tasc-conformance-fixed-wing.json` |
| A-005 | Version-impact map reflects current taxonomy/schema versions and check coverage. | `THM-TASC-DATA-001` | Required for deterministic compatibility gating and versioned release decisions. | `tools/data_policy/version_impact_gate.py` | `conformance/reports/data-version-impact.json` |
| A-006 | Branch protection remains enabled with required gate contexts on the release branch. | `THM-TASC-GOV-001` | Required to prevent bypass of mandatory CI/conformance/release controls. | `tools/governance/policy_gate.py` | `governance/branch-protection.status.yaml` |

## Review Cadence

- Assumptions are reviewed at each release rehearsal.
- Any invalidated assumption requires:
- a risk acceptance entry or immediate remediation
- updated theorem and traceability linkage
