# ASC/TASC Governance Charter

## Mission

Maintain ASC kernel safety semantics and TASC assurance contracts as an auditable, release-gated system suitable for procurement, underwriting, and regulatory inquiry.

## Scope

- Normative safety semantics in `ASC.md` and `spec/asc/*`.
- TASC assurance checks, schemas, policies, and verifier behavior in `spec/tasc/*`, `schemas/*`, and `tools/tasc-verify/*`.
- Release gating and evidence packaging in `.github/workflows/*`, `tools/hashlock/*`, and `tools/releasepack/*`.

## Authority Model

- `Safety Lead` has veto authority on Class A safety semantics and invariants.
- `Security Lead` has veto authority on TA2, PKI, transparency, and revocation controls.
- `Release Manager` has authority to cut or block GA based on release criteria.
- `Program Manager` has tie-break authority only after Safety and Security sign-off is recorded.

## Decision Cadence

- Weekly governance triage (open risks, exceptions, pending Class A changes).
- Bi-weekly architecture review for kernel/assurance interface impacts.
- Monthly release-readiness review with must-pass matrix status.

## Escalation Matrix

| Trigger | Escalation Owner | Initial Response SLA | Resolution SLA |
| --- | --- | --- | --- |
| Required CI gate failure on `main` | Release Manager | 4 hours | 2 business days |
| TA2 / transparency / revocation failure | Security Lead | 2 hours | 24 hours |
| Kernel invariant regression | Safety Lead | 2 hours | 24 hours |
| Unresolved Class A approval conflict | Program Manager | 1 business day | 5 business days |

## Quorum and Approval

- Class A: Safety Lead + Security Lead + Release Manager approvals required.
- Class B: owning WG lead + one independent reviewer.
- Class C: code owner approval and passing CI.

## Required Artifacts

- Class A requires a decision record in `governance/DECISION_RECORDS/`.
- Risk exceptions require a record in `governance/RISK_ACCEPTANCE_POLICY.md` format.
- Meeting notes for governance decisions must be archived in `governance/MEETING_NOTES/`.
