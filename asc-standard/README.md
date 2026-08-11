# TASC Product Surface

TASC is now presented as a product entry surface, not a repo tour.

The first-run path is:

`doctor` -> `init` -> `verify` -> `pack` -> `explain`

The governing promise is:

`same input + same profile bundle + same versioned contract surface = same verdict + same report shape`

## Start Here

Run all commands from `asc-standard/`.

```bash
./tasc doctor --operation verify
./tasc init demo
./tasc verify examples/minimal-local
./tasc explain last
```

`./tasc pack` is wired in, but it still depends on the underlying assurance-pack toolchain and current trust artifacts being fresh.

If you only read one thing first, read this README and then choose one lane below.

## Audience Lanes

- Integrator: `start/integrator.md`
- Implementer: `start/implementer.md`
- Operator: `start/operator.md`
- Auditor: `start/auditor.md`

## Happy Path

- `./tasc doctor`
  Checks toolchain, trust mode, config origins, and whether the current operation requires networked services.
- `./tasc init demo`
  Copies a minimal example workspace.
- `./tasc verify <path>`
  Verifies a bundle or normalizes an existing verifier report into the stable failure contract.
- `./tasc pack <path>`
  Wraps the assurance-pack path behind trust-mode defaults.
- `./tasc explain last`
  Emits grouped, stable failure records in human, JSON, or SARIF form.
- `./tasc lock`
  Captures pinned profile and contract-surface versions for CI.
- `./tasc check-lock`
  Fails if pinned lock values drift from `tasc.yaml`.
- `./tasc check-example <path>`
  Verifies an example and compares normalized output to `expected-report.json`.
- `./tasc ci-preflight`
  Runs doctor, lock drift checks, and golden example checks in one gate.
  Use `--strict` to fail on doctor warnings and `--output <json>` to emit a CI artifact.

## Contracts

- Workspace config: `tasc.yaml`
- Lockfile: `tasc.lock.yaml`
- Profile bundles: `profile-bundles/*.yaml`
- Stable failure schema: `docs/handbook/FAILURE_SCHEMA.md`
- Support bundles: `docs/handbook/SUPPORT_BUNDLES.md`
- Plugin/extension contract: `docs/handbook/PLUGIN_CONTRACT.md`
- Upgrade and rollback policy: `docs/handbook/UPGRADE_AND_ROLLBACK.md`
- Generated product docs: `docs/generated/`

## Public Integration Surface

Treat these two files as the external boundary:

- `spec/interfaces/api.openapi.yaml`
- `reference/contracts/interfaces.v1.yaml`

Everything else is either reference implementation, internal tooling, or governance/evidence machinery.

## Minimal Examples

- `examples/minimal-local`
- `examples/minimal-signed`
- `examples/minimal-replay`

These examples are CI-backed product invariants for the current install surface.

## What Not To Read Yet

Do not start with:

- `docs/runbooks/`
- `governance/`
- `safety-case/`
- `conformance/vectors/`
- `evidence/manifests/`

Those areas matter later, but they are not the first-run UX.

## Existing Deep Docs

- Handbook: `docs/handbook/README.md`
- Tutorials: `docs/tutorials/README.md`
- Runbooks: `docs/runbooks/README.md`
- Release criteria: `RELEASE_CRITERIA.md`

## CI Workflows

- `sdk-rust`: Rust SDK fmt/clippy/test/demo checks (`.github/workflows/sdk-rust.yml`).
