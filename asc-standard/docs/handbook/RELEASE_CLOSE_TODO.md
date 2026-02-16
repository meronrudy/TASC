# Release Close TODO (GA)

Date: 2026-02-16
Branch: `ga-capability-closure`

## 1. Traceability Blocker

- [x] Resolve `tracecheck` missing evidence references.
- [x] Move legacy evidence links to canonical/fixture-backed paths in `safety-case/traceability/test_to_evidence.csv`.
- [x] Re-run `python3 tools/tracecheck/tracecheck.py --repo-root .`.

## 2. Commit Scope and Churn

- [x] Remove transient generated artifacts (`__pycache__`, `conformance/mirror-log`, `dist/.handoff-stage`, temp perf bundles).
- [x] Keep only intentional source/policy/spec/tooling changes and canonical release artifacts.
- [x] Update ignore rules to prevent recurring transient churn (`.gitignore`).

## 3. Hard-Gate Matrix (CI-Equivalent Local)

- [x] Governance/data gates.
  - `tools/data_policy/version_impact_gate.py`
  - `tools/governance/policy_gate.py`
  - `tools/governance/class_a_gate.py`
- [x] Traceability + replay drift + release guard.
  - `tools/tracecheck/tracecheck.py`
  - `replay-drift-validator`
  - `tools/releasepack/release_guard.py`
- [x] Strict TA2 + dual transparency verification for all profiles.
  - `assurancepack.py` live publish (`rekor`,`mirror`)
  - `tasc-verify verify --require-ta TA2 --require-transparency rekor,mirror`
  - `offline_smoke.py` parity check

## 4. Deterministic Packaging Check

- [x] Run canonicalization + hashlock + releasepack.
- [x] Repeat with unchanged inputs.
- [x] Verify no-diff for:
  - `evidence/manifests/releasepack.json`
  - `evidence/manifests/hashlock.json`
  - `evidence/manifests/releasepack.tgz`

## 5. Handoff Package

- [x] Regenerate `dist/tasc-handoff-ga.tgz`.
- [x] Validate wrapper smoke in clean temp extraction.

## 6. Host-Side Branch Protection (Manual)

- [ ] Confirm repository hosting protections require:
  - `ci`
  - `conformance`
  - `kernel-ci`
  - `release`

## 7. Final Cut (Manual Operator Step)

- [ ] Final commit with clean tree.
- [ ] Create GA tag.
- [ ] Publish:
  - `evidence/manifests/releasepack.tgz`
  - `evidence/manifests/releasepack.json`
  - `dist/tasc-handoff-ga.tgz`
