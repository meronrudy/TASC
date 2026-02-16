# Runbook: Key Compromise

Use this runbook when there is evidence or suspicion that a signing key/certificate used for attestation or logs is compromised.

## 1. Trigger Conditions

- key material exfiltration signal,
- unauthorized signature observed,
- certificate misuse or issuance anomaly,
- failed integrity checks indicating signer identity mismatch.

## 2. Immediate Containment

1. Halt signing operations using the suspected key.
2. Block affected key identity in operational signing workflows.
3. Open security incident ticket and notify security lead.
4. Snapshot all related trust artifacts and logs.

## 3. Revoke and Rotate

1. Update revocation source for compromised serial/key.
2. Regenerate and publish updated revocation snapshot:
   - `policies/attestation/revocation-snapshot.json`
3. Rotate signer key/certificate in managed signing infrastructure.
4. Update trust material as required:
   - `policies/attestation/pki/ta2-signer.cert.pem`
   - `policies/attestation/pki/ta2-chain.pem`
   - `policies/attestation/pki/trust-roots.pem` (only if root/intermediate changed)

## 4. Re-Issue Affected Artifacts

For each impacted profile:

1. regenerate assurance pack,
2. regenerate verifier report,
3. re-run offline smoke,
4. refresh hashlock/releasepack manifests.

```bash
python3 tools/assurancepack/assurancepack.py --repo-root . --profile <profile> --assurance-pack-version 0.3 --lifecycle-stage active --output <pack> --archive <archive> --badge-id <badge> --signer-mode pkcs11 --pkcs11-profile policies/attestation/pkcs11-profile.yaml --pkcs11-module "$PKCS11_MODULE" --pkcs11-token-label tasc-soft-token --pkcs11-key-label tasc-ta2-key --pkcs11-cert-label tasc-ta2-cert --pkcs11-pin-env TASC_PKCS11_PIN --pkcs11-mechanism SHA256-RSA-PKCS --publish-live --rekor-url https://rekor.sigstore.dev --mirror-url http://127.0.0.1:17777
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify --bundle <pack> --profile <profile> --policy eu-north-star --require-ta TA2 --require-transparency rekor,mirror
python3 tools/tasc-verify/offline_smoke.py --repo-root . --profiles uas-small,fixed-wing,hybrid-vtol
python3 tools/hashlock/hashlock.py --repo-root .
python3 tools/releasepack/releasepack.py --repo-root .
```

## 5. Badge and Registry Actions

If compromise impacts badge trustworthiness:

1. mark affected badge entries revoked/suspended in registry,
2. publish updated badge status,
3. reissue badge only after successful re-verification.

Artifacts:

- `policies/badge-registry.json`
- governance decision records in `governance/DECISION_RECORDS/`

## 6. Exit Criteria

Containment is complete when:

- compromised key is revoked and out of service,
- new signer path is active,
- all required profile bundles re-verified PASS,
- release manifests rebuilt,
- governance/security sign-offs captured.
