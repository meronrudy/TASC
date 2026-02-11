# Verifier Reference

This guide documents `tasc-verify` and offline smoke behavior.

## 1. CLI Commands

### `verify`

Validates a full assurance pack and emits deterministic JSON output.

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack-uas-small.json \
  --profile uas-small \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror \
  --output evidence/manifests/tasc-conformance-uas-small.json
```

Exit behavior:

- `0` only when all required checks PASS.
- non-zero when any required check fails.

### `report`

Summarizes an existing verifier JSON output.

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- report \
  --input evidence/manifests/tasc-conformance-uas-small.json
```

### `check-proof`

Validates a standalone transparency proof payload.

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- check-proof \
  --proof /tmp/rekor-proof.json \
  --trusted-checkpoints policies/transparency/trusted-log-checkpoints.json
```

## 2. Policy and Trust Inputs

`verify` supports explicit trust/policy path overrides:

- `--checks-file`
- `--trusted-checkpoints`
- `--badge-registry`
- `--attestation-trust-policy`
- `--trust-roots`
- `--revocation-snapshot`
- `--freshness-policy`
- `--transparency-policy`
- `--remediation-file`

Use explicit absolute paths when invoking from external working directories.

## 3. Offline Smoke Script

`tools/tasc-verify/offline_smoke.py` mirrors the same check IDs and major verifier semantics for fixture bundles without Rust compilation.

```bash
python3 tools/tasc-verify/offline_smoke.py \
  --repo-root . \
  --profiles uas-small,fixed-wing,hybrid-vtol \
  --output conformance/reports/tasc-offline-smoke.json
```

Expected output:

- one per-profile PASS/FAIL summary,
- aggregate result,
- deterministic JSON report.

## 4. Check Categories

Check IDs are defined in `spec/tasc/checks.yaml`.

Primary categories include:

- schema envelope and artifact schema checks,
- profile/policy match checks,
- topology mediation/exclusivity checks,
- artifact hash integrity,
- signed log hash-chain/signature checks,
- retention and incident-window compliance checks,
- replay completeness,
- TA2 attestation checks,
- dual transparency proof checks,
- badge active/not-revoked checks.

Remediation mapping:

- `spec/tasc/remediation.yaml`
- `docs/handbook/CHECK_REMEDIATION.md`

## 5. Failure Triage Flow

1. Inspect `failedChecks` in verifier output JSON.
2. Map each check to remediation guidance.
3. Regenerate affected artifacts (do not hand-edit conformance output).
4. Re-run `verify` and `offline_smoke`.
5. Re-run `hashlock` and `releasepack` if manifests changed.

## 6. Determinism Expectations

Verifier output is deterministic given the same:

- bundle payload,
- policy/trust inputs,
- check ID catalog.

Timestamp-sensitive freshness checks depend on wall-clock age and configured max-age policy.
