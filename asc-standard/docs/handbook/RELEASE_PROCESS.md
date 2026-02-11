# Release Process

This document defines the end-to-end release flow for a single GA push.

## 1. Preconditions

- All required branch protections are configured host-side for `ci`, `conformance`, `kernel-ci`, `release`.
- `RELEASE_CRITERIA.md` sign-off owners are known.
- Schema/check ID freeze policy is active for the GA train.
- Trust/policy files are current and committed.

## 2. Build and Validation Sequence

### Step 1: Spec and Code Consistency

```bash
for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/specgen/Cargo.toml -- --profile "$p" --repo-root .
done
cargo test --manifest-path tools/specgen/Cargo.toml
```

### Step 2: Kernel + Deterministic Replay

```bash
cargo test --manifest-path reference/kernel/Cargo.toml --workspace
cargo run --manifest-path reference/kernel/Cargo.toml -p asc-conformance-kernel --bin replay-drift-validator -- \
  --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol --output conformance/reports/replay-drift.json
```

### Step 3: Build Assurance Packs

```bash
for p in uas-small fixed-wing hybrid-vtol; do
  python3 tools/assurancepack/assurancepack.py \
    --repo-root . --profile "$p" \
    --output "evidence/manifests/tasc-assurance-pack-${p}.json" \
    --archive "evidence/manifests/tasc-assurance-pack-${p}.tgz" \
    --badge-id "badge-${p}-active" \
    --signer-mode pkcs11 \
    --pkcs11-sign-cmd "openssl dgst -sha256 -sign policies/attestation/pki/ta2-signer.key.pem -out {output} {input}"
done
```

### Step 4: Verify Assurance Packs (Rust + Offline)

```bash
for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
    --bundle "evidence/manifests/tasc-assurance-pack-${p}.json" \
    --profile "$p" --policy eu-north-star --require-ta TA2 --require-transparency rekor,mirror \
    --output "evidence/manifests/tasc-conformance-${p}.json"
done

python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol
```

### Step 5: Policy and Provenance Gates

```bash
python3 tools/data_policy/version_impact_gate.py --repo-root . --output evidence/manifests/data-version-impact.json
python3 tools/governance/policy_gate.py --repo-root . --output evidence/manifests/governance-policy-gate.json
python3 tools/tracecheck/tracecheck.py --repo-root .
python3 tools/hashlock/hashlock.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

## 3. Required Output Artifacts

Minimum expected outputs before tagging:

- `conformance/reports/replay-drift.json`
- `conformance/reports/tasc-offline-smoke.json`
- `evidence/manifests/tasc-assurance-pack-*.json`
- `evidence/manifests/tasc-conformance-*.json`
- `evidence/manifests/hashlock.json`
- `evidence/manifests/releasepack.json`
- `evidence/manifests/releasepack.tgz`

## 4. Rehearsal Model

Internal dry runs should execute the exact release flow with no manual post-processing of generated evidence.

Recommended structure:

1. Rehearsal A: full run, collect findings.
2. Rehearsal B: full run after fixes.
3. Final GA run: no known open findings, all gates green.

## 5. Failure Handling

If a required check fails:

1. Stop release pipeline.
2. Capture failing check IDs and command output.
3. Apply remediation from `docs/handbook/CHECK_REMEDIATION.md`.
4. Re-run from the earliest affected stage.
5. Re-generate downstream manifests to restore lineage integrity.

## 6. Post-Cut Validation

After release tag:

- Re-run `tasc-verify verify` against tagged release artifacts.
- Re-run offline smoke against tagged fixtures.
- Archive final sign-off matrix and reports with release metadata.
