#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 2 ]; then
  echo "usage: verify-offline.sh <bundle.json> <profile>" >&2
  exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PKG_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUNDLE="$1"
PROFILE="$2"
BUNDLE_ABS="$(cd "$(dirname "$BUNDLE")" && pwd)/$(basename "$BUNDLE")"
BUNDLE_DIR="$(cd "$(dirname "$BUNDLE_ABS")" && pwd)"

# Deterministic handoff tarballs set mtime=0; refresh local snapshot mtime for
# strict freshness gates that also validate file age.
touch "$PKG_ROOT/policies/attestation/revocation-snapshot.json"
find "$PKG_ROOT/sample" "$PKG_ROOT/policies" -type f \( -name "*.pem" -o -name "revocation-snapshot.json" \) -exec touch {} +

cd "$BUNDLE_DIR"
"$PKG_ROOT/bin/tasc-verify" verify   --bundle "$BUNDLE_ABS"   --profile "$PROFILE"   --policy eu-north-star   --require-ta TA2   --require-transparency rekor,mirror   --checks-file "$PKG_ROOT/spec/tasc/checks.yaml"   --trusted-checkpoints "$PKG_ROOT/policies/transparency/trusted-log-checkpoints.json"   --badge-registry "$PKG_ROOT/policies/badge-registry.json"   --attestation-trust-policy "$PKG_ROOT/policies/attestation/trust-policy.json"   --trust-roots "$PKG_ROOT/policies/attestation/pki/trust-roots.pem"   --revocation-snapshot "$PKG_ROOT/policies/attestation/revocation-snapshot.json"   --freshness-policy "$PKG_ROOT/policies/provenance/freshness-policy.yaml"   --transparency-policy "$PKG_ROOT/policies/transparency/verification-policy.yaml"   --remediation-file "$PKG_ROOT/spec/tasc/remediation.yaml"
