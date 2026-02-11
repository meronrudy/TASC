# Check Remediation Matrix

Generated from `spec/tasc/checks.yaml` and `spec/tasc/remediation.yaml`.

Use this as the first triage reference when `tasc-verify` returns failed checks.

## Triage Procedure

1. Run verifier and capture output JSON.
2. Read `failedChecks` in report.
3. Apply remediation actions below.
4. Regenerate affected artifacts.
5. Re-run `verify` and offline smoke.
6. Rebuild downstream manifests (`hashlock`, `releasepack`) if artifacts changed.

## Matrix

| Check ID | Category | Remediation |
| --- | --- | --- |
| `CHK_SCHEMA_ASSURANCEPACK` | `schema` | Ensure assurance pack contains all required top-level sections and valid JSON object structure. |
| `CHK_SCHEMA_EVIDENCEMAP` | `schema` | Regenerate EvidenceMap from current topology and include required sub-sections. |
| `CHK_SCHEMA_CONFORMANCE_REPORT` | `schema` | Re-run tasc-verify and embed the verifier-produced report without manual edits. |
| `CHK_SCHEMA_SIGNED_LOG` | `schema` | Emit signedOperationalLog with schemaVersion, signerKeyId, signature, root, and events. |
| `CHK_SCHEMA_REPLAY_RECIPE` | `schema` | Include all replay digests and recipeVersion in replayRecipe. |
| `CHK_SCHEMA_ATTESTATION` | `schema` | Provide complete attestationEvidence payload matching schema requirements. |
| `CHK_SCHEMA_INCIDENT_INITIAL` | `schema` | Include incidentInitialTemplate with required reporting windows fields. |
| `CHK_SCHEMA_INCIDENT_PACK` | `schema` | Include incidentPackTemplate with requiredArtifacts and fullPackSlaDays. |
| `CHK_SCHEMA_TRANSPARENCY_REKOR` | `schema` | Include transparencyProofs.rekor with required proof fields. |
| `CHK_SCHEMA_TRANSPARENCY_MIRROR` | `schema` | Include transparencyProofs.mirror with required proof fields. |
| `CHK_SCHEMA_BADGE_ENTRY` | `schema` | Include badgeEntry with badgeId, badgeType, and active status fields. |
| `CHK_PROFILE_MATCH` | `policy` | Regenerate bundle with matching `--profile` and evidence profile metadata. |
| `CHK_POLICY_MATCH` | `policy` | Set bundle policy to match verifier invocation policy. |
| `CHK_TOPOLOGY_INTERLOCK_MEDIATION` | `topology` | Ensure every actuator path flows through `interlock_gate`. |
| `CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY` | `topology` | Ensure only `safety_kernel` sources authority edges into `interlock_gate`. |
| `CHK_TOPOLOGY_ASSERTION_REFS` | `topology` | Populate A001/A002 assertions as proven with `sha256` proofRef values. |
| `CHK_ARTIFACT_HASH_INTEGRITY` | `integrity` | Recompute artifact digests and ensure files exist beside bundle. |
| `CHK_LOG_HASH_CHAIN` | `integrity` | Rebuild event chain so `prevHash`/`hash`/`root` are internally consistent. |
| `CHK_LOG_SIGNATURE` | `integrity` | Re-sign operational log root with declared signer identity and trusted chain. |
| `CHK_RETENTION_EU_MINIMUM` | `compliance` | Set `evidenceMap.logs.retentionPolicy.minimumMonths` to at least `6`. |
| `CHK_REPLAY_COMPLETENESS` | `replay` | Populate seeds/build/config/environment/container digests and profile. |
| `CHK_ATTESTATION_TA2` | `attestation` | Provide TA2 attestation meeting trust policy, revocation, lifecycle, and digest-binding constraints. |
| `CHK_INCIDENT_WINDOWS_EU` | `compliance` | Set incident windows to `15/2/10` and `fullPackSlaDays <= 10`. |
| `CHK_TRANSPARENCY_REKOR_PROOF` | `transparency` | Regenerate Rekor proof against trusted checkpoint, signature, freshness, and digest-binding rules. |
| `CHK_TRANSPARENCY_MIRROR_PROOF` | `transparency` | Regenerate mirror proof against trusted checkpoint, signature, freshness, and parity rules. |
| `CHK_BADGE_ACTIVE_NOT_REVOKED` | `governance` | Use active badge entry present as active in badge registry. |
