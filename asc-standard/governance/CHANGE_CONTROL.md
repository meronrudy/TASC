# Change Control

## Change Classes

- `Class A`:
- Safety invariants, authority mediation, TA2/trust floor, transparency proof policy, check ID semantics.
- `Class B`:
- New checks, non-breaking schema additions, conformance vector expansion, tooling logic changes without interface break.
- `Class C`:
- Documentation, comments, non-functional refactors with no contract change.

## Required Artifacts by Class

- `Class A`:
- Decision record (`governance/DECISION_RECORDS/DR-*.md`)
- risk assessment or exception entry
- evidence impact statement (checks/tests/evidence manifests affected)
- rollback plan
- `Class B`:
- PR description with impact mapping
- updated tests/vectors as applicable
- `Class C`:
- standard PR description and passing CI

## Review and Approval SLA

| Class | Review Start SLA | Review Completion SLA | Approvers |
| --- | --- | --- | --- |
| A | 1 business day | 5 business days | Safety Lead + Security Lead + Release Manager |
| B | 1 business day | 3 business days | Owning WG Lead + independent reviewer |
| C | 1 business day | 2 business days | Code owner |

## Emergency Path

- Emergency Class A changes may bypass normal cadence only for active safety/security incidents.
- Emergency approvals still require Safety Lead and Security Lead sign-off.
- Post-incident retrospective and permanent decision record required within 2 business days.
