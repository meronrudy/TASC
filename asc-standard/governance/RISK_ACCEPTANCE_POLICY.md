# Risk Acceptance Policy

## Purpose

Define the minimum controls for accepting residual risk when a release gate cannot be satisfied in full.

## Required Fields

- `Risk ID`
- `Owner`
- `Affected release scope`
- `Safety/Security impact`
- `Compensating controls`
- `Expiry date`
- `Revalidation trigger`

## Exception SLA

- New risk acceptance request triage: 1 business day.
- Decision on acceptance/rejection: 3 business days.
- Expired risk acceptance remediation or extension decision: 1 business day.

## Approval Authority

- Safety risks: Safety Lead approval required.
- Security/trust risks: Security Lead approval required.
- Release gating risks: Release Manager approval required.
- Cross-domain risks: joint approval from all three roles.

## Expiry and Renewal

- Risk acceptances expire by default in 30 days.
- Renewal requires updated evidence and explicit justification of unresolved root cause.
- Expired items automatically block GA cut until resolved or renewed.
