# ASC Tutorials

This tutorial set is intended for a new engineer to go from spec edit to release-grade assurance outputs.

All commands assume working directory `asc-standard/`.

## Tutorial 1: Spec Change -> Generated Artifacts

Goal: make a safe spec change and regenerate deterministic artifacts.

1. Edit one normative value in `spec/asc/*.yaml`.
2. Regenerate for a profile:

```bash
cargo run --manifest-path tools/specgen/Cargo.toml -- --profile uas-small --repo-root .
```

3. Inspect generated outputs:

- `reference/kernel/crates/asc-types/src/generated_reason_codes.rs`
- `reference/kernel/crates/asc-kernel-model/src/generated_thresholds.rs`
- `reference/kernel/crates/asc-kernel-model/src/generated_profile.rs`
- `evidence/manifests/spec-hash.txt`

4. Validate build health:

```bash
cargo test --manifest-path tools/specgen/Cargo.toml
cargo test --manifest-path reference/kernel/Cargo.toml --workspace
```

Expected result: tests pass and generated files only change where the spec changed.

## Tutorial 2: Deterministic Replay Validation

Goal: prove replay stability and drift behavior.

```bash
cargo run --manifest-path reference/kernel/Cargo.toml -p asc-conformance-kernel --bin replay-drift-validator -- \
  --repo-root . \
  --profiles uas-small,fixed-wing,hybrid-vtol \
  --output conformance/reports/replay-drift.json
```

Inspect report:

- `conformance/reports/replay-drift.json`

Expected result:

- same-seed runs have stable verdict and tip-hash,
- drift policy behavior matches fixture expectations,
- overall report result is `PASS`.

## Tutorial 3: Build and Verify TASC Assurance Packs

Goal: produce procurement-grade artifacts and verify them.

1. Build one pack:

```bash
python3 tools/assurancepack/assurancepack.py \
  --repo-root . \
  --profile uas-small \
  --output evidence/manifests/tasc-assurance-pack-uas-small.json \
  --archive evidence/manifests/tasc-assurance-pack-uas-small.tgz \
  --badge-id badge-uas-small-active \
  --signer-mode pkcs11 \
  --pkcs11-sign-cmd "openssl dgst -sha256 -sign policies/attestation/pki/ta2-signer.key.pem -out {output} {input}"
```

2. Verify with Rust CLI:

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack-uas-small.json \
  --profile uas-small --policy eu-north-star \
  --require-ta TA2 --require-transparency rekor,mirror \
  --output evidence/manifests/tasc-conformance-uas-small.json
```

3. Summarize report:

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- report \
  --input evidence/manifests/tasc-conformance-uas-small.json
```

Expected result: verifier PASS and empty `failedChecks`.

## Tutorial 4: Offline Smoke Across All Profiles

Goal: verify all profile fixtures without Rust compilation.

```bash
python3 tools/tasc-verify/offline_smoke.py \
  --repo-root . \
  --profiles uas-small,fixed-wing,hybrid-vtol \
  --output conformance/reports/tasc-offline-smoke.json
```

Expected result:

- each profile PASS,
- overall PASS,
- report generated at `conformance/reports/tasc-offline-smoke.json`.

## Tutorial 5: Full Provenance and Release Packaging

Goal: produce final release evidence manifests.

```bash
python3 tools/data_policy/version_impact_gate.py --repo-root . --output evidence/manifests/data-version-impact.json
python3 tools/governance/policy_gate.py --repo-root . --output evidence/manifests/governance-policy-gate.json
python3 tools/tracecheck/tracecheck.py --repo-root .
python3 tools/hashlock/hashlock.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

Inspect final outputs:

- `evidence/manifests/tracecheck-report.json`
- `evidence/manifests/hashlock.json`
- `evidence/manifests/releasepack.json`
- `evidence/manifests/releasepack.tgz`

Expected result: complete lineage and freshness checks pass with no missing required checks.

## Tutorial 6: Failing Check Triage

Goal: debug and remediate failed verifier checks.

1. Open report JSON and inspect `failedChecks`.
2. Map each ID to `docs/handbook/CHECK_REMEDIATION.md`.
3. Regenerate affected artifacts.
4. Re-run `verify` and `offline_smoke`.
5. Re-run `hashlock` and `releasepack` for updated evidence.

Typical checks to triage first:

- topology checks,
- attestation/trust checks,
- transparency proof checks,
- retention/incident policy checks.
