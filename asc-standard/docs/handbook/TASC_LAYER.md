# TASC Layer Integration Handbook

This handbook explains how the additive TASC layer is integrated over ASC core without renaming ASC kernel components.

## 1. Integration Objective

TASC is treated as a contract-enforced evidence interface, not as a replacement runtime architecture.

It answers three external questions:

1. What did the system do?
2. What guardrails were active?
3. What can be independently proven after the fact?

The acceptance object is the `AssurancePack` and its verifier result.

## 2. Required Artifacts

A release-grade TASC assurance pack includes:

- `EvidenceMap`
- `ConformanceReport`
- `SignedOperationalLog`
- `ReplayRecipe`
- `AttestationEvidence`
- `IncidentInitial` template
- `IncidentPack` template
- `TransparencyProof` entries for `rekor` and `mirror`
- `BadgeEntry`
- signed procurement objects:
  - Shipment Eligibility Certificate
  - Underwriter Confidence Packet
  - Procurement Bid Packet
  - Recycler Intake Passport (conditional)

Schemas are under `schemas/` and checks are in `spec/tasc/checks.yaml`.

## 3. Verifier Expectations

GA verification requires:

- policy `eu-north-star`,
- attestation floor `TA2`,
- both transparency proofs (`rekor`, `mirror`),
- active/non-revoked badge,
- procurement object section (`assurancePackVersion 0.3`) with signed standalone parity,
- all required check IDs PASS.

Canonical command:

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack-uas-small.json \
  --profile uas-small \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror \
  --output evidence/manifests/tasc-conformance-uas-small.json
```

Offline parity validation:

```bash
python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol
```

## 4. Topology Integration Rules

TASC topology checks enforce ASC boundary semantics:

- every actuator path must be mediated by `interlock_gate`,
- authority into `interlock_gate` is exclusive to `safety_kernel`,
- `EvidenceMap.topology.conformanceAssertions` must include proven assertion references for these rules.

These map directly to check IDs:

- `CHK_TOPOLOGY_INTERLOCK_MEDIATION`
- `CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY`
- `CHK_TOPOLOGY_ASSERTION_REFS`

## 5. Trust and Cryptographic Requirements

### Attestation

- TA2-level requirements enforced by verifier against trust policy and revocation snapshot.
- Chain digest and evidence digest are bound and validated.
- Signing-time and revocation-window temporal semantics are enforced.

Key files:

- `policies/attestation/trust-policy.json`
- `policies/attestation/revocation-snapshot.json`
- `policies/attestation/pki/`

### Transparency

- Both `rekor` and `mirror` proofs required.
- Signature, chain, checkpoint hash, digest shape, freshness, and optional parity constraints are verified.

Key files:

- `policies/transparency/verification-policy.yaml`
- `policies/transparency/trusted-log-checkpoints.json`

## 6. EU North-Star Baseline

Policy baseline captures critical expectations used by multinational buyers/insurers:

- logging capability and retention floor (>= 6 months),
- incident reporting field readiness for 15/2/10-day windows,
- deterministic post-incident replay and evidence production.

Mapped checks include:

- `CHK_RETENTION_EU_MINIMUM`
- `CHK_INCIDENT_WINDOWS_EU`
- `CHK_REPLAY_COMPLETENESS`

## 7. Sidecar-First Migration (Legacy Systems)

Recommended sequence:

1. Add adapter shims around existing control outputs.
2. Place interlock gate between planner outputs and actuator integrator.
3. Start in evidence-only mode to generate maps/logs/replay artifacts.
4. Enable default-closed and heartbeat enforcement.
5. Tighten envelope and authority policy under staged acceptance tests.
6. Map legacy artifacts to schema-compatible evidence fields.

## 8. Procurement, Underwriting, and Governance Surfaces

- Contract clauses: `clauses/rfp.md`, `clauses/msa.md`, `clauses/sow.md`
- Underwriting + incident templates: `templates/`
- Badge governance: `policies/badge-policy.md`, `policies/badge-registry.json`

These surfaces are designed so third parties can verify with the same CLI used in CI.

## 9. Troubleshooting

Use check IDs as the primary debugging index.

1. Run `tasc-verify verify` and capture `failedChecks`.
2. Map IDs in `docs/handbook/CHECK_REMEDIATION.md`.
3. Regenerate affected artifacts.
4. Re-run both Rust verifier and offline smoke.
5. Rebuild provenance manifests if evidence changed.
