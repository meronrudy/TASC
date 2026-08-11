# ASC Handbook

This handbook is now downstream of the product entry surface in `README.md`.

Use the handbook when you already know your lane and need the deeper operating model.

## Start With Product Docs

- Product landing page: `README.md`
- Integrator lane: `start/integrator.md`
- Implementer lane: `start/implementer.md`
- Operator lane: `start/operator.md`
- Auditor lane: `start/auditor.md`

## Operational Contracts

- Workspace config: `docs/handbook/CONFIG_CONTRACT.md`
- Rust SDK contract: `docs/handbook/SDK_CONTRACT.md`
- Failure schema and exit codes: `docs/handbook/FAILURE_SCHEMA.md`
- Support bundles and redaction: `docs/handbook/SUPPORT_BUNDLES.md`
- Plugin and extension contract: `docs/handbook/PLUGIN_CONTRACT.md`
- Upgrade and rollback: `docs/handbook/UPGRADE_AND_ROLLBACK.md`

## Existing Deep References

- Architecture and boundaries: `docs/handbook/ARCHITECTURE.md`
- TASC layer and migration details: `docs/handbook/TASC_LAYER.md`
- Verifier reference: `docs/handbook/VERIFIER_REFERENCE.md`
- Release process: `docs/handbook/RELEASE_PROCESS.md`
- Release readiness backlog: `docs/handbook/RELEASE_READINESS_TODO.md`
- Check remediation: `docs/handbook/CHECK_REMEDIATION.md`

## Command Reference

Product-facing:

```bash
./tasc doctor --operation verify
./tasc init demo
./tasc verify examples/minimal-local
./tasc explain last
./tasc support-bundle examples/minimal-local
./tasc check-lock --format json
./tasc check-example examples/minimal-local
./tasc ci-preflight --format json --strict --output .tasc/ci-preflight-summary.json
```

Deep repo-facing:

```bash
cargo run --manifest-path tools/specgen/Cargo.toml -- --profile uas-small --repo-root .
cargo test --manifest-path reference/kernel/Cargo.toml --workspace
python3 tools/assurancepack/assurancepack.py --repo-root . --profile uas-small --output evidence/manifests/tasc-assurance-pack-uas-small.json --archive evidence/manifests/tasc-assurance-pack-uas-small.tgz
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify --bundle evidence/manifests/tasc-assurance-pack-uas-small.json --profile uas-small --policy eu-north-star --require-ta TA2 --require-transparency rekor,mirror --output evidence/manifests/tasc-conformance-uas-small.json
```

## What Not To Read Yet

- `docs/runbooks/`
- `governance/`
- `safety-case/`
- `conformance/`

Read those only when your lane requires them.
