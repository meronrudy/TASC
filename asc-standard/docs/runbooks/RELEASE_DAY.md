# Runbook: Release Day

Use this runbook for final GA cut and signing.

## 1. Pre-Flight Checklist

- Branch is up to date and required CI workflows are green.
- No unresolved critical findings from rehearsal runs.
- `RELEASE_CRITERIA.md` owners are available for sign-off.
- Host-side branch protections are enabled for required gates.

Run wrapper preflight before deep regeneration:

```bash
./tasc doctor --operation verify --format json
./tasc check-lock --format json
./tasc check-example examples/minimal-local
./tasc ci-preflight --format json
```

## 2. Clean Regeneration

Run complete deterministic regeneration and validation sequence from scratch:

```bash
for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/specgen/Cargo.toml -- --profile "$p" --repo-root .
done

cargo test --manifest-path reference/kernel/Cargo.toml --workspace
cargo run --manifest-path reference/kernel/Cargo.toml -p asc-conformance-kernel --bin replay-drift-validator -- --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol --output conformance/reports/replay-drift.json

python3 tools/releasepack/canonicalize.py --repo-root . --manifest-dir evidence/manifests --allowlist evidence/canonical-artifacts.yaml --output evidence/manifests/canonicalize-report.json
python3 tools/hashlock/hashlock.py --repo-root .

for p in uas-small fixed-wing hybrid-vtol; do
  python3 tools/assurancepack/assurancepack.py \
    --repo-root . --profile "$p" \
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

for p in uas-small fixed-wing hybrid-vtol; do
  cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
    --bundle "evidence/manifests/tasc-assurance-pack-${p}.json" \
    --profile "$p" --policy eu-north-star --require-ta TA2 --require-transparency rekor,mirror
done

python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol
python3 tools/data_policy/version_impact_gate.py --repo-root . --output evidence/manifests/data-version-impact.json
python3 tools/governance/policy_gate.py --repo-root . --output evidence/manifests/governance-policy-gate.json
python3 tools/tracecheck/tracecheck.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

`assurancepack.py` writes `tasc-conformance-<profile>.json` directly. Avoid re-writing those files with `verify --output` or rotating `hashlock.json` between assurance-pack generation and `releasepack.py`, or lineage validation will fail.

## 3. Release Artifact Checklist

Confirm presence and validity of:

- `conformance/reports/replay-drift.json`
- `conformance/reports/tasc-offline-smoke.json`
- `evidence/manifests/tasc-assurance-pack-*.json`
- `evidence/manifests/tasc-conformance-*.json`
- `evidence/manifests/hashlock.json`
- `evidence/manifests/releasepack.json`
- `evidence/manifests/releasepack.tgz`

## 4. Sign-Off and Tag

1. Collect sign-offs from release criteria owners.
2. Confirm no pending critical changes.
3. Create release tag and publish release notes.

## 5. Rollback Criteria

Abort release if any of these occur:

- any required check failure,
- freshness/provenance gate failure,
- unresolved trust integrity issue,
- missing required sign-off.

Rollback action:

- stop tag/publish,
- document failure cause,
- fix and rerun full release day sequence.
