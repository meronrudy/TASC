# DR-20260211-GA-CAPABILITY-CLOSURE

- ID: DR-20260211-GA-CAPABILITY-CLOSURE
- Date: 2026-02-11
- Owner: ASC/TASC Maintainers
- Scope: GA capability closure for TA2 attestation, dual transparency, replay operational parity, governance automation, and release artifact canonicalization.
- Decision:
  Adopt the stricter GA verification baseline as mandatory for all release-bound profiles (`uas-small`, `fixed-wing`, `hybrid-vtol`) with hard-fail gates in CI and release workflows.
- Safety impact:
  Raises assurance confidence by requiring machine-verifiable evidence integrity, real trust-policy checks, and deterministic replay parity checks.
- Rollback/mitigation:
  If a gate blocks production due to tooling regressions, ship a hotfix that restores verifier/tool parity without lowering TA2/transparency policy floors.
- Linked evidence:
  `spec/tasc/checks.yaml`, `tools/tasc-verify/src/main.rs`, `tools/tasc-verify/offline_smoke.py`, `conformance/reports/*`, `evidence/manifests/*`
