# Runbook: Incident Response

Use this runbook for runtime safety events or serious assurance anomalies.

## 1. Trigger Conditions

Start this runbook when any of the following occur:

- safety interlock clamp or shutdown in operational mission,
- unexpected actuator command denial or authority violation,
- verifier-detected integrity anomaly for an active mission evidence stream,
- event meeting internal serious incident criteria.

## 2. Immediate Actions (0-30 Minutes)

1. Stabilize operations (safe-mode, mission hold, or controlled termination).
2. Preserve evidence artifacts without modification.
3. Capture incident timestamp, profile, system identifier, and operator context.
4. Notify incident response contact and safety lead.

## 3. Build Initial Incident Report

Use template `templates/incident-initial.md` and emit JSON payload with:

- system/profile context,
- severity,
- narrative,
- evidence references:
  - EvidenceMap hash,
  - Signed log segment hash,
  - Replay recipe hash,
  - Attestation hash,
- reporting windows (15/2/10 baseline fields).

Reference runtime helper:

- `reference/supervisor/runtime.py` (`build_incident_initial`).

## 4. Evidence Collection Checklist

Collect these immutable references:

- assurance pack used for mission window,
- verifier conformance report,
- signed operational log segment and root,
- transparency proofs for both logs,
- attestation evidence and trust material snapshot,
- replay inputs and environment digests,
- related lineage manifests (`hashlock`, `releasepack`).

## 5. Full Incident Pack (Audit-Ready)

Assemble full pack using `templates/incident-pack.md` including:

- initial report reference,
- full artifact references,
- corrective action plan hash,
- investigation notes and containment actions,
- recovery/rollback decision.

Reference runtime helper:

- `reference/supervisor/runtime.py` (`build_incident_pack`).

## 6. Verification Before External Submission

Validate evidence integrity before sharing with insurer/regulator:

```bash
cargo run --manifest-path tools/tasc-verify/Cargo.toml -- verify \
  --bundle <incident_assurance_pack.json> \
  --profile <profile> \
  --policy eu-north-star \
  --require-ta TA2 \
  --require-transparency rekor,mirror
```

If verifier fails, document failing checks and include remediation status in incident notes.

## 7. Post-Incident Closure

1. Record root cause and corrective action.
2. Re-run conformance and replay drift for affected profile(s).
3. Rebuild assurance artifacts and manifests.
4. File governance records for safety-critical changes.
5. Update risk acceptance artifacts if exceptions are introduced.
