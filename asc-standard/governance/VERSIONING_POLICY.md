# Versioning Policy

## Semantic Versioning Rules

- `ASC` core specs and kernel contracts use SemVer: `MAJOR.MINOR.PATCH`.
- `TASC` public schemas and check identifiers are externally consumed contracts and follow SemVer with explicit migration notes.
- Check IDs in `spec/tasc/checks.yaml` are immutable identifiers for GA; behavior may tighten only with Class A approval.

## Compatibility Requirements

- `0.1.x` assurance artifacts remain readable for one minor cycle after `0.2.x` release.
- Backward read compatibility is mandatory for verifier CLI in the active GA train.
- Breaking schema changes require:
- Class A approval
- migration playbook update
- release-note compatibility section

## Data Taxonomy Version Impact

- Changes in `data/incident-taxonomy/*`, `data/logging-schema/*`, `data/replay/*`, or `data/retention-policy.md` must update `data/version-impact-map.yaml`.
- `tools/data_policy/version_impact_gate.py` is a hard CI gate and must pass on PR/release.

## Class A Freeze Window

- During GA freeze, schema field removals, check ID changes, and trust policy floor changes are blocked unless:
- Safety Lead + Security Lead + Release Manager jointly approve
- an emergency rationale is documented in a decision record
- rollback path is documented
