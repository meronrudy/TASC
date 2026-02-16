# Procurement Bid Packet (Template)

## Contract Deliverable Name
Procurement Bid Packet

## Required Fields
- objectType: `procurement_bid_packet`
- objectName: `Procurement Bid Packet`
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
