# Architecture Guide

This document describes the runtime and evidence architecture from core kernel outward.

## 1. Core Runtime Layers

### Layer A: ASC Kernel (Rust)

The kernel evaluates safety constraints and emits a deterministic `KernelOutput`:

- Inputs: state, intent, tick metadata.
- Checks: frame validity, bounds, flow limits, energy, temporal guarantees, invariants.
- Arbitration: highest-severity verdict precedence (`Allow < Clamp < Hold < Override < Shutdown`).
- Output fields: verdict, reasons, constrained command, contract fingerprint.

Key crates:

- `reference/kernel/crates/asc-kernel-model`
- `reference/kernel/crates/asc-kernel-runtime`
- `reference/kernel/crates/asc-types`
- `reference/kernel/crates/asc-logging`

### Layer B: Authority Boundary + Interlock (Reference Surface)

Outside the kernel, runtime authority and enforcement boundaries are represented by:

- Adapter normalization (`reference/adapters/runtime.py`): canonicalizes external control requests to kernel intent shape and enforces safety-kernel authority source.
- Interlock gate (`reference/interlock/runtime.py`): default-closed logic, heartbeat checks, envelope checks, latch behavior.
- Supervisor (`reference/supervisor/runtime.py`): incident initial/full pack orchestration with required evidence references.

### Layer C: Evidence and Assurance

TASC artifacts convert runtime behavior into verifiable acceptance objects:

- `EvidenceMap`
- `ConformanceReport`
- `SignedOperationalLog`
- `ReplayRecipe`
- `AttestationEvidence`
- `IncidentInitial` / `IncidentPack`
- `TransparencyProof` (both logs)
- `BadgeEntry`
- `AssurancePack` (envelope)

## 2. Trust and Verification Boundaries

### Attestation Boundary

- TA2 minimum is required for GA acceptance.
- Verification binds signer certificate/chain/trust roots and revocation snapshot.
- Verifier enforces issuer/chain constraints, nonce policy, key lifecycle policy, signing-time checks, and evidence digest binding.

Policy files:

- `policies/attestation/trust-policy.json`
- `policies/attestation/revocation-snapshot.json`
- `policies/attestation/pki/*.pem`

### Transparency Boundary

- Dual proof requirement: `rekor` and `mirror`.
- Both proofs must validate checkpoint binding, inclusion/consistency digest form, signer chain, payload signature, freshness, and (if configured) entryDigest parity.

Policy files:

- `policies/transparency/verification-policy.yaml`
- `policies/transparency/trusted-log-checkpoints.json`

### Governance Boundary

- Badge must be active in both bundle and registry.
- Safety-critical change flows are governed by Class A controls and policy gates.

Policy files:

- `policies/badge-policy.md`
- `policies/badge-registry.json`
- `governance/`

## 3. Determinism and Replay

Determinism is enforced in three places:

1. Kernel check/arbitration behavior (Rust tests).
2. Replay drift validator (`replay-drift-validator`) using fixed seeds and profile fixtures.
3. TASC replay metadata + bundle verification checks.

Primary report:

- `conformance/reports/replay-drift.json`

## 4. Provenance Chain

Expected lineage order:

1. Spec hash
2. Conformance report
3. Assurance pack
4. Hashlock manifest
5. Releasepack manifest/archive

Freshness and completeness are checked against:

- `policies/provenance/freshness-policy.yaml`

## 5. Data Flow Summary

1. Spec edit (`spec/asc`, `spec/tasc`, `spec/profiles`).
2. Generated artifacts (`tools/specgen`).
3. Runtime/conformance execution.
4. Assurance pack assembly (`tools/assurancepack`).
5. Verifier checks (`tools/tasc-verify` + offline smoke).
6. Provenance packaging (`tracecheck`, `hashlock`, `releasepack`).
7. CI/release gates.

## 6. Profile Model

The same assurance contract applies to all required profiles:

- `uas-small`
- `fixed-wing`
- `hybrid-vtol`

Profile-specific vectors and fixtures live in:

- `conformance/vectors/`
- `conformance/fixtures/`

Acceptance requires PASS across all profiles.
