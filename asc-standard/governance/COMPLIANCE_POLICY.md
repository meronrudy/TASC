# Compliance Policy

## Baseline

Every merge must include traceability and evidence artifacts relevant to changed requirements.

## Mandatory Gates

- `ci`, `conformance`, `kernel-ci`, and `release` workflows must remain hard-fail.
- `tools/tasc-verify` checks must pass for all three profiles with:
- `--policy eu-north-star`
- `--require-ta TA2`
- `--require-transparency rekor,mirror`
- Replay drift validator must pass (`reference/kernel` replay-drift-validator).
- Data version-impact compatibility gate must pass (`tools/data_policy/version_impact_gate.py`).
- Release placeholder guard must pass (`tools/releasepack/release_guard.py`).

## Class A Requirements

- Decision record present and linked in PR.
- Risk acceptance recorded when any gate is waived (temporary waiver only).
- Explicit owner + due date for remediation.

## Auditability

- Conformance and release reports must be retained in `conformance/reports` and `evidence/manifests`.
- Governance artifacts must be version-controlled and immutable after release tag.
