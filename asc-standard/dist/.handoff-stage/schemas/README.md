# TASC Schemas

Canonical machine-readable contracts for all assurance artifacts.

## Schema Set

- `assurance-pack.schema.json`
- `evidence-map.schema.json`
- `conformance-report.schema.json`
- `signed-operational-log.schema.json`
- `replay-recipe.schema.json`
- `attestation-evidence.schema.json`
- `incident-initial.schema.json`
- `incident-pack.schema.json`
- `transparency-proof.schema.json`
- `badge-entry.schema.json`
- `badge-registry.schema.json`
- `procurement-object.schema.json`

## Contract Role

These schemas define the expected structure of procurement/underwriting acceptance artifacts and are consumed by:

- assurance pack generation,
- verifier checks,
- offline smoke verification,
- policy/provenance packaging flows.

## Versioning Expectations

- Treat schema versions as public contracts.
- Keep check IDs in `spec/tasc/checks.yaml` stable across a GA train.
- Any non-backward-compatible schema change requires governance review and migration notes.
- `assurance-pack.schema.json` `0.3` adds signed procurement objects and lifecycle context.
- `0.2` Assurance Packs remain readable for one minor cycle, but GA policy checks require the `0.3` procurement object section.
