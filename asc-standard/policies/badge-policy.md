# TASC Badge Policy v0.1

## Badge Types

- `TASC_CONFORMANT`: baseline conformance evidence.
- `TASC_AUDIT_GRADE`: attestation-backed, insurer-ready conformance.

## Issuance Criteria

- `tasc-verify` report result is `PASS`.
- Required topology assertions are `proven` with hash-addressed proof references.
- Dual transparency proofs (`rekor`, `mirror`) validate against trusted checkpoints.
- Attestation meets minimum `TA2` requirements.

## Surveillance

- A new badge entry is required for each safety-relevant release.
- Quarterly surveillance required for `TASC_AUDIT_GRADE`.
- Annual comprehensive review required for all active badges.

## Revocation Triggers

- Key compromise or unverifiable signature chains.
- Proven material misrepresentation in assurance artifacts.
- Repeated failure to meet replay or incident response SLAs.
- Missing or invalid transparency proofs in declared mandatory logs.

## Registry Model

Badge entries and revocation events are tracked in a machine-readable registry and are verifier-checkable offline.
