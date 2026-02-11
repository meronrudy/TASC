# ASC Handbook

This handbook is the source-of-truth operations guide for engineering, assurance, governance, and release execution.

All commands assume working directory `asc-standard/`.

## 1. What This System Produces

ASC Standard produces two coupled outputs:

1. Runtime safety behavior from the ASC kernel and surrounding authority boundaries.
2. A deterministic, machine-verifiable TASC assurance object (`AssurancePack`) suitable for procurement, underwriting, and audit workflows.

The assurance object is accepted only if `tasc-verify` reports full PASS on required checks under GA policy constraints.

## 2. Operating Model (Spec to Release)

1. Update normative specs in `spec/asc/` and profile definitions in `spec/profiles/`.
2. Run `tools/specgen` to regenerate derived Rust artifacts and refresh `evidence/manifests/spec-hash.txt`.
3. Run kernel tests and replay determinism checks.
4. Build TASC assurance packs.
5. Verify packs with Rust verifier and offline smoke.
6. Run data/governance policy gates.
7. Build provenance artifacts (`tracecheck`, `hashlock`, `releasepack`).
8. Ensure all CI gates pass and required branch protections are active.

## 3. Command Reference (Canonical)

```bash
# Spec generation
cargo run --manifest-path tools/specgen/Cargo.toml -- --profile uas-small --repo-root .

# Kernel + replay determinism
cargo test --manifest-path reference/kernel/Cargo.toml --workspace
cargo run --manifest-path reference/kernel/Cargo.toml -p asc-conformance-kernel --bin replay-drift-validator -- \
  --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol --output conformance/reports/replay-drift.json

# Assurance pack generation
python3 tools/assurancepack/assurancepack.py \
  --repo-root . --profile uas-small \
  --output evidence/manifests/tasc-assurance-pack-uas-small.json \
  --archive evidence/manifests/tasc-assurance-pack-uas-small.tgz \
  --badge-id badge-uas-small-active \
  --signer-mode pkcs11 \
  --pkcs11-sign-cmd "openssl dgst -sha256 -sign policies/attestation/pki/ta2-signer.key.pem -out {output} {input}"

# Rust verifier
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle evidence/manifests/tasc-assurance-pack-uas-small.json \
  --profile uas-small --policy eu-north-star \
  --require-ta TA2 --require-transparency rekor,mirror \
  --output evidence/manifests/tasc-conformance-uas-small.json

# Offline smoke verifier (no Rust compile)
python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol

# Policy/provenance gates
python3 tools/data_policy/version_impact_gate.py --repo-root . --output evidence/manifests/data-version-impact.json
python3 tools/governance/policy_gate.py --repo-root . --output evidence/manifests/governance-policy-gate.json
python3 tools/tracecheck/tracecheck.py --repo-root .
python3 tools/hashlock/hashlock.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

## 4. Hard GA Defaults

- Required profiles: `uas-small`, `fixed-wing`, `hybrid-vtol`
- Required policy: `eu-north-star`
- Required attestation floor: `TA2`
- Required transparency proofs: `rekor`, `mirror`
- Required badge state: active/not revoked
- Required retention baseline: >= 6 months
- Required incident windows baseline: 15/2/10 days (standard/widespread/fatal)

See `RELEASE_CRITERIA.md` for sign-off and gate ownership.

## 5. Evidence Contracts

Primary acceptance objects:

- `evidence/manifests/tasc-assurance-pack-<profile>.json`
- `evidence/manifests/tasc-conformance-<profile>.json`
- `conformance/reports/tasc-offline-smoke.json`
- `evidence/manifests/hashlock.json`
- `evidence/manifests/releasepack.json`
- `evidence/manifests/releasepack.tgz`

If any required check fails, the bundle is rejected.

## 6. Reference Components Outside Kernel

Runnable components and tests:

- Adapter: `reference/adapters/runtime.py`
- Interlock: `reference/interlock/runtime.py`
- Supervisor: `reference/supervisor/runtime.py`
- Integration tests: `python3 -m unittest reference.tests.test_reference_surface`

These components model authority mediation, default-closed interlock behavior, and incident-pack orchestration.

## 7. Where To Go Next

- Architecture and boundaries: `docs/handbook/ARCHITECTURE.md`
- TASC policy and migration details: `docs/handbook/TASC_LAYER.md`
- Verifier commands and check behavior: `docs/handbook/VERIFIER_REFERENCE.md`
- Release execution flow: `docs/handbook/RELEASE_PROCESS.md`
- Check-by-check remediation guidance: `docs/handbook/CHECK_REMEDIATION.md`
- End-to-end tutorials: `docs/tutorials/README.md`
- Operational runbooks: `docs/runbooks/`
