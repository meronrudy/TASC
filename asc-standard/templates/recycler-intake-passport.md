# Recycler Intake Passport (Template)

## Contract Deliverable Name
Recycler Intake Passport

## Required Fields
- objectType: `recycler_intake_passport`
- objectName: `Recycler Intake Passport`
- policyPackId
- policyPackVersion
- inputHash
- bundleDigest
- signer (identity + trustAnchorLevel)
- validity.notBeforeUtc / validity.notAfterUtc
- verifierInstructions.command / verifierInstructions.requiredChecks
- signatureEnvelope.payloadDigest / signatureEnvelope.signedAtUtc / signatureEnvelope.signature
- artifactRef.path / artifactRef.sha256
- inputs

## Conditional Requirement
Required only when lifecycle stage is `decommission` or `recycle`.
