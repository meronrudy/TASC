# assurancepack

Builds a profile-specific TASC Assurance Pack JSON and signed archive.

## Usage

```bash
python3 tools/assurancepack/assurancepack.py --repo-root . --profile uas-small \
  --output evidence/manifests/tasc-assurance-pack.json \
  --archive evidence/manifests/tasc-assurance-pack.tgz \
  --signer-mode pkcs11 \
  --pkcs11-profile policies/attestation/pkcs11-profile.yaml \
  --pkcs11-module /usr/lib/softhsm/libsofthsm2.so \
  --pkcs11-token-label tasc-soft-token \
  --pkcs11-key-label tasc-ta2-key \
  --pkcs11-cert-label tasc-ta2-cert \
  --pkcs11-pin-env TASC_PKCS11_PIN \
  --pkcs11-mechanism SHA256-RSA-PKCS \
  --publish-live \
  --rekor-url https://rekor.sigstore.dev \
  --mirror-url http://127.0.0.1:17777 \
  --trust-roots policies/attestation/pki/trust-roots.pem \
  --revocation-snapshot policies/attestation/revocation-snapshot.json
```

For non-TA2 local development only, file signing remains available:

```bash
python3 tools/assurancepack/assurancepack.py --repo-root . --profile uas-small \
  --require-ta TA1 \
  --signer-mode file \
  --signing-key policies/attestation/pki/ta2-signer.key.pem \
  --signing-cert policies/attestation/pki/ta2-signer.cert.pem \
  --signing-chain policies/attestation/pki/ta2-chain.pem
```

For conformance fixtures:

```bash
python3 tools/assurancepack/assurancepack.py --repo-root . --profile fixed-wing \
  --output conformance/fixtures/tasc/tasc-assurance-pack-fixed-wing.json \
  --archive conformance/fixtures/tasc/tasc-assurance-pack-fixed-wing.tgz
```

The tool also emits a sidecar signature file `<archive>.sig`.

`conformanceReport` is verifier-generated (`tasc-verify`) and embedded into the
assurance pack after checks pass.

For TA2 builds, `--publish-live` is mandatory and dual transparency proofs are
embedded from live Rekor + mirror publication responses.
