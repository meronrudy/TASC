# Remediation Catalog

Generated from `spec/tasc/remediation.yaml`.

## CHK_SCHEMA_ASSURANCEPACK

Ensure assurance pack contains all required top-level sections and valid JSON object structure.

## CHK_SCHEMA_EVIDENCEMAP

Regenerate EvidenceMap from current topology and include required sub-sections.

## CHK_SCHEMA_CONFORMANCE_REPORT

Re-run tasc-verify and embed the verifier-produced report without manual edits.

## CHK_SCHEMA_SIGNED_LOG

Emit signedOperationalLog with schemaVersion, signerKeyId, signature, root, and events.

## CHK_SCHEMA_REPLAY_RECIPE

Include all replay digests and recipeVersion in replayRecipe.

## CHK_SCHEMA_ATTESTATION

Provide complete attestationEvidence payload matching schema requirements.

## CHK_SCHEMA_INCIDENT_INITIAL

Include incidentInitialTemplate with required reporting windows fields.

## CHK_SCHEMA_INCIDENT_PACK

Include incidentPackTemplate with requiredArtifacts and fullPackSlaDays.

## CHK_SCHEMA_TRANSPARENCY_REKOR

Include transparencyProofs.rekor with required proof fields.

## CHK_SCHEMA_TRANSPARENCY_MIRROR

Include transparencyProofs.mirror with required proof fields.

## CHK_SCHEMA_BADGE_ENTRY

Include badgeEntry with badgeId, badgeType, and active status fields.

## CHK_SCHEMA_PROCUREMENT_OBJECTS

Add procurementObjects section with all required fields defined in schemas/procurement-object.schema.json.

## CHK_PROCUREMENT_OBJECT_REQUIRED_SET

Include Shipment Eligibility Certificate, Underwriter Confidence Packet, Procurement Bid Packet, and conditionally Recycler Intake Passport.

## CHK_PROCUREMENT_OBJECT_ARTIFACT_PARITY

Regenerate standalone procurement object artifacts and ensure artifactRef digests/paths match embedded copies.

## CHK_PROCUREMENT_OBJECT_INPUT_HASH

Recompute inputHash from canonicalized inputs and update the signed payload.

## CHK_PROCUREMENT_OBJECT_BUNDLE_BINDING

Regenerate procurement objects after final bundle digest is known so bundleDigest fields are accurate.

## CHK_PROCUREMENT_OBJECT_SIGNATURE

Re-sign procurement object payloads with declared signer material and verify certificate paths exist.

## CHK_PROCUREMENT_OBJECT_TRUST_FLOOR

Ensure each procurement object signer declares TA2 and uses PKCS11-backed key source for GA policy.

## CHK_PROCUREMENT_OBJECT_VALIDITY_WINDOW

Set notBeforeUtc/notAfterUtc to a valid active interval at verification time.

## CHK_RECYCLER_INTAKE_CONDITIONAL

Provide Recycler Intake Passport when lifecycleStage is decommission or recycle; omit otherwise.

## CHK_PROFILE_MATCH

Regenerate bundle with matching --profile and evidence profile metadata.

## CHK_POLICY_MATCH

Set bundle policy to match verifier invocation policy.

## CHK_TOPOLOGY_INTERLOCK_MEDIATION

Ensure every actuator path flows through interlock_gate.

## CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY

Ensure only safety_kernel sources authority edges into interlock_gate.

## CHK_TOPOLOGY_ASSERTION_REFS

Populate A001/A002 assertions as proven with sha256 proofRef values.

## CHK_ARTIFACT_HASH_INTEGRITY

Recompute artifact digests and ensure files exist beside bundle.

## CHK_LOG_HASH_CHAIN

Rebuild event chain so prevHash/hash/root are internally consistent.

## CHK_LOG_SIGNATURE

Re-sign operational log root with declared signerKeyId.

## CHK_RETENTION_EU_MINIMUM

Set evidenceMap.logs.retentionPolicy.minimumMonths to at least 6.

## CHK_REPLAY_COMPLETENESS

Populate seeds/build/config/environment/container digests and profile.

## CHK_REPLAY_OPERATIONAL_PARITY

Re-run runtime mission + replay-from-log so verdict/reasons/tip-hash drift_count is zero.

## CHK_ATTESTATION_TA2

Provide TA2 attestation meeting trust policy and lifecycle constraints.

## CHK_INCIDENT_WINDOWS_EU

Set incident windows to 15/2/10 and fullPackSlaDays <= 10.

## CHK_TRANSPARENCY_REKOR_PROOF

Regenerate Rekor proof against trusted checkpoint and signature rules.

## CHK_TRANSPARENCY_MIRROR_PROOF

Regenerate mirror proof against trusted checkpoint and signature rules.

## CHK_BADGE_ACTIVE_NOT_REVOKED

Use active badge entry present as active in badge registry.
