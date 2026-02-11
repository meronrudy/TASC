# DO-178C Compliance Map

This map links ASC/TASC artifacts to DO-178C objective families using public FAA AC 20-115D framing.

| DO-178C objective family | ASC/TASC artifact(s) | Verification evidence | Trace IDs |
| --- | --- | --- | --- |
| Planning and standards | `spec/asc/*.yaml`, `spec/tasc/checks.yaml` | Spec hash + change control records | MAP-178C-001 |
| Requirements traceability | `safety-case/traceability/*.csv` | `tools/tracecheck/tracecheck.py` output | MAP-178C-002 |
| Verification independence and repeatability | `tools/tasc-verify`, `tools/tasc-verify/offline_smoke.py` | Deterministic conformance report checks | MAP-178C-003 |
| Configuration management baseline | `evidence/manifests/hashlock.json`, `evidence/manifests/spec-hash.txt` | Hash-integrity checks and drift gates | MAP-178C-004 |
| Problem reporting / corrective action | `templates/incident-*`, `incidentInitialTemplate`, `incidentPackTemplate` | CHK_INCIDENT_WINDOWS_EU + incident pack generation | MAP-178C-005 |

## Linked Runtime Tests
- `reference/kernel/crates/asc-conformance-kernel/tests/determinism.rs`
- `reference/kernel/crates/asc-conformance-kernel/tests/precedence.rs`
- `reference/kernel/crates/asc-conformance-kernel/tests/runtime_from_repo.rs`
