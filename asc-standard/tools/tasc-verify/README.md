# tasc-verify

Verifier CLI for TASC Assurance Pack artifacts.

## Commands

### verify

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack.json \
  --profile uas-small \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror \
  --attestation-trust-policy policies/attestation/trust-policy.json \
  --trust-roots policies/attestation/pki/trust-roots.pem \
  --revocation-snapshot policies/attestation/revocation-snapshot.json \
  --freshness-policy policies/provenance/freshness-policy.yaml \
  --transparency-policy policies/transparency/verification-policy.yaml \
  --remediation-file spec/tasc/remediation.yaml \
  --output evidence/manifests/tasc-conformance-report.json
```

Returns non-zero when required checks fail.
Check messages include remediation guidance from the remediation file.

### report

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- report \
  --input evidence/manifests/tasc-conformance-report.json
```

### check-proof

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- check-proof \
  --proof evidence/manifests/transparency-rekor-proof.json
```

## Check Source of Truth

Required check IDs and order are defined in `spec/tasc/checks.yaml`.

## Offline smoke validation (no Rust compilation)

Run the Python smoke validator to validate all fixture bundles (`uas-small`, `fixed-wing`, `hybrid-vtol`) with the same check IDs:

```bash
python3 tools/tasc-verify/offline_smoke.py --repo-root .
```

This writes a combined report to:

- `conformance/reports/tasc-offline-smoke.json`

and returns non-zero if any required check fails for any profile.

## Negative vectors (hard gate)

Run negative vectors that intentionally break each required check ID and verify
that `tasc-verify` reports the expected failure:

```bash
cargo build --manifest-path tools/tasc-verify/Cargo.toml
python3 tools/tasc-verify/negative_vectors.py \
  --repo-root . \
  --vectors-index conformance/vectors/tasc-negative-index.yaml \
  --bundle-template conformance/fixtures/tasc/tasc-assurance-pack-{profile}.json \
  --profiles uas-small,fixed-wing,hybrid-vtol \
  --verifier-bin tools/tasc-verify/target/debug/tasc-verify
```

Output report:

- `conformance/reports/tasc-negative-vectors.json`

## Troubleshooting matrix generation

Generate handbook troubleshooting content from canonical check/remediation specs:

```bash
python3 tools/tasc-verify/generate_troubleshooting.py --repo-root .
```
