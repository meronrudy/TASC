# ASC Standard Monorepo

ASC Standard is a spec-first reference implementation for a safety kernel (`ASC`) plus an additive audit-grade assurance layer (`TASC`).

The repository is designed for two simultaneous outcomes:

1. Deterministic, enforceable runtime safety behavior.
2. Procurement/underwriting-ready evidence artifacts that can be independently verified offline.

## Release Baseline (GA)

- Core naming: `ASC` remains unchanged.
- Assurance naming: `TASC` remains additive.
- Required profiles: `uas-small`, `fixed-wing`, `hybrid-vtol`.
- Required policy: `eu-north-star`.
- Required trust floor: `TA2`.
- Required transparency proofs: `rekor`, `mirror`.
- Required verifier posture: hard fail on any failed required check.

Full gate contract is documented in `RELEASE_CRITERIA.md`.

## Repository Layout

- `spec/`: normative ASC and TASC requirements.
- `schemas/`: machine-verifiable contracts (assurance pack, evidence map, logs, replay, attestation, incidents, transparency, badge).
- `reference/kernel/`: Rust ASC kernel crates and replay drift validator.
- `reference/adapters`, `reference/interlock`, `reference/supervisor`: runnable reference surface outside kernel.
- `tools/specgen`: spec-to-generated Rust constants/types and spec hash generation.
- `tools/assurancepack`: assurance bundle builder and signer integration.
- `tools/tasc-verify`: verifier CLI (`verify`, `report`, `check-proof`).
- `tools/hashlock`, `tools/releasepack`, `tools/tracecheck`: provenance and packaging controls.
- `policies/`: attestation, transparency, freshness, badge policy and registry.
- `conformance/`: suites, vectors, fixtures, and generated reports.
- `evidence/manifests/`: canonical generated evidence and release packaging outputs.
- `docs/`: handbook, tutorials, and runbooks.

## Prerequisites

Run all commands from `asc-standard/`.

- Rust toolchain with `cargo`.
- Python 3.
- `openssl` (used for signature and chain validation).
- `jq` (optional but useful for inspecting JSON outputs).

## Quickstart (Single Profile)

```bash
cargo run --manifest-path tools/specgen/Cargo.toml -- --profile uas-small --repo-root .
cargo test --manifest-path reference/kernel/Cargo.toml --workspace

python3 tools/assurancepack/assurancepack.py \
  --repo-root . \
  --profile uas-small \
  --assurance-pack-version 0.3 \
  --lifecycle-stage active \
  --output evidence/manifests/tasc-assurance-pack-uas-small.json \
  --archive evidence/manifests/tasc-assurance-pack-uas-small.tgz \
  --badge-id badge-uas-small-active \
  --signer-mode pkcs11 \
  --pkcs11-profile policies/attestation/pkcs11-profile.yaml \
  --pkcs11-module "$PKCS11_MODULE" \
  --pkcs11-token-label tasc-soft-token \
  --pkcs11-key-label tasc-ta2-key \
  --pkcs11-cert-label tasc-ta2-cert \
  --pkcs11-pin-env TASC_PKCS11_PIN \
  --pkcs11-mechanism SHA256-RSA-PKCS \
  --publish-live \
  --rekor-url https://rekor.sigstore.dev \
  --mirror-url http://127.0.0.1:17777

cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack-uas-small.json \
  --profile uas-small \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror \
  --output evidence/manifests/tasc-conformance-uas-small.json
```

## Full Release Pipeline (All Profiles)

```bash
# 1) Spec generation and hash pinning
for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/specgen/Cargo.toml -- --profile "$p" --repo-root .
done

# 2) Kernel + conformance replay determinism
cargo test --manifest-path reference/kernel/Cargo.toml --workspace
cargo run --manifest-path reference/kernel/Cargo.toml -p asc-conformance-kernel --bin replay-drift-validator -- \
  --repo-root . \
  --profiles uas-small,fixed-wing,hybrid-vtol \
  --output conformance/reports/replay-drift.json

# 3) Build assurance packs for all profiles
for p in uas-small fixed-wing hybrid-vtol; do
  python3 tools/assurancepack/assurancepack.py \
    --repo-root . \
    --profile "$p" \
    --assurance-pack-version 0.3 \
    --lifecycle-stage active \
    --output "evidence/manifests/tasc-assurance-pack-${p}.json" \
    --archive "evidence/manifests/tasc-assurance-pack-${p}.tgz" \
    --badge-id "badge-${p}-active" \
    --signer-mode pkcs11 \
    --pkcs11-profile policies/attestation/pkcs11-profile.yaml \
    --pkcs11-module "$PKCS11_MODULE" \
    --pkcs11-token-label tasc-soft-token \
    --pkcs11-key-label tasc-ta2-key \
    --pkcs11-cert-label tasc-ta2-cert \
    --pkcs11-pin-env TASC_PKCS11_PIN \
    --pkcs11-mechanism SHA256-RSA-PKCS \
    --publish-live \
    --rekor-url https://rekor.sigstore.dev \
    --mirror-url http://127.0.0.1:17777
done

# 4) Verify each pack via Rust verifier
for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
    --bundle "evidence/manifests/tasc-assurance-pack-${p}.json" \
    --profile "$p" \
    --policy eu-north-star \
    --require-ta TA2 \
    --require-transparency rekor,mirror \
    --output "evidence/manifests/tasc-conformance-${p}.json"
done

# 5) Verify same check IDs without Rust compilation
python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol

# 6) Policy gates + provenance packaging
python3 tools/data_policy/version_impact_gate.py --repo-root . --output evidence/manifests/data-version-impact.json
python3 tools/governance/policy_gate.py --repo-root . --output evidence/manifests/governance-policy-gate.json
python3 tools/tracecheck/tracecheck.py --repo-root .
python3 tools/hashlock/hashlock.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

## CI Gates

Mandatory workflows:

- `ci`
- `conformance`
- `kernel-ci`
- `release`

These workflows hard fail on:

- failed kernel/conformance/replay checks,
- failed TASC checks,
- missing required check coverage,
- policy/freshness/provenance gate failures,
- placeholder/starter scaffolding in release-bound assets.

## Documentation Index

- Handbook entrypoint: `docs/handbook/README.md`
- Architecture guide: `docs/handbook/ARCHITECTURE.md`
- TASC integration details: `docs/handbook/TASC_LAYER.md`
- Verifier reference: `docs/handbook/VERIFIER_REFERENCE.md`
- Release process: `docs/handbook/RELEASE_PROCESS.md`
- Check remediation: `docs/handbook/CHECK_REMEDIATION.md`
- Tutorials: `docs/tutorials/README.md`
- Runbooks: `docs/runbooks/`
- Release criteria: `RELEASE_CRITERIA.md`

## Support Surfaces

- Contract clauses: `clauses/rfp.md`, `clauses/msa.md`, `clauses/sow.md`
- Operator/underwriting templates: `templates/`
- Safety-case mappings: `safety-case/`
