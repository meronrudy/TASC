# Stub and Scaffold Analysis

This snapshot lists remaining scaffold indicators and release-readiness gaps after current implementation pass.

## Resolved in this pass
- Placeholder conformance text removed from generated assurance packs.
- Starter compliance maps replaced with mapping tables.
- README-only conformance suites now have concrete suite manifests.
- `reference/adapters`, `reference/interlock`, and `reference/supervisor` now include runnable implementations and integration tests.
- Offline smoke now mirrors verifier check IDs with trust/freshness/transparency policy inputs.
- Deterministic replay drift is validated with executable kernel runtime replay checks and profile fixtures.
- Data taxonomy/schema compatibility is now hard-gated via version-impact CI checks.
- Release-domain `.gitkeep` ambiguity is removed with explicit scope READMEs.

## Remaining scaffold indicators
- None in release-critical scope.

## Rust release-readiness inventory (current)
- Implemented and hard-validated:
  - `tools/tasc-verify` builds clean with clippy, has executable integration tests for `verify`, `report`, and `check-proof`, and passes offline smoke parity checks across all three profiles.
  - Kernel workspace (`reference/kernel`) builds clean with clippy and passes determinism/precedence/runtime integration tests.
  - Event log hashing path no longer panics on JSON serialization edge cases and now has explicit tests.
  - `tools/specgen` now has validation-focused unit coverage for baseline, required reason-code enforcement, and timing constraint enforcement.
  - `asc-kernel-model` and `asc-kernel-runtime` now include direct unit tests for dynamic limit enforcement paths.
  - `asc-types` now has explicit verdict precedence unit coverage.
  - `asc-conformance-kernel` now has unit coverage for profile mapping, deterministic noise generation, and CLI argument parsing behavior.
- Remaining Rust gaps (non-blocking but not yet fully hardened):
  - None identified in release-critical Rust paths after current pass.

## Partial implementation hotspots
- Governance:
  - Host-side branch protection is now applied and tracked via `governance/branch-protection.status.yaml`.

## Recommended completion order
1. Maintain replay drift fixture updates only through governed release flow.
2. Continue expanding proof automation in `proofs/proof-ci/`.
