# TASC Policies

Policy artifacts consumed by verifier and packaging gates.

## Core Policy Files

- `badge-policy.md`
  - Badge issuance, surveillance cadence, and revocation triggers.
- `badge-registry.json`
  - Active/revoked badge status checked by verifier.
- `attestation/trust-policy.json`
  - TA-level constraints, issuer allowlist, chain digest constraints, nonce/key usage requirements.
- `attestation/revocation-snapshot.json`
  - Offline revocation state and validity window.
- `provenance/freshness-policy.yaml`
  - Max-age and lineage-ordering constraints for generated evidence.
- `transparency/verification-policy.yaml`
  - Required logs, freshness limits, bundle digest binding, mirror parity requirements.
- `transparency/trusted-log-checkpoints.json`
  - Trusted checkpoints and signer trust material for each required transparency log.
- `procurement/object-policy.yaml`
  - Required signed procurement object set, conditional Recycler rule, and validity defaults.
- `procurement/object-signer-profile.yaml`
  - Per-object signer identity/profile configuration used by assurance pack generation.

## PKI Material

`attestation/pki/` includes local trust roots, signer cert/chain, and key material for reference/testing flows.

Production deployments should replace private signing keys with managed HSM/PKCS#11 integration while preserving verifier trust contracts.
