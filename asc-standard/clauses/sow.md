# TASC SOW Acceptance Criteria v0.1

## Deliverables Per Milestone

1. TASC Assurance Pack (`schemas/assurance-pack.schema.json` compliant)
2. Passing `tasc-verify` report
3. Replay SOP + SLA declaration
4. Incident templates and response contacts

## Acceptance Tests

1. Schema compliance of Assurance Pack and all sub-artifacts
2. Topology interlock mediation and authority exclusivity checks
3. Artifact hash integrity verification
4. Log chain + signature verification
5. TA2 attestation validation
6. Dual transparency proof validation (`rekor`, `mirror`)
7. Badge active/not-revoked validation

## Non-Conformance Handling

Supplier must remediate failed checks at no additional cost and resubmit a passing bundle.
