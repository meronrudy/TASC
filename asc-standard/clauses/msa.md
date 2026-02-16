# TASC MSA Language v0.3

## Safety Assurance Artifacts

Vendor shall deliver a TASC Assurance Pack for every release and safety-relevant configuration.

Vendor shall also deliver named signed procurement objects (embedded in the assurance pack and as standalone artifacts):
- Shipment Eligibility Certificate
- Underwriter Confidence Packet
- Procurement Bid Packet
- Recycler Intake Passport (conditional for `decommission`/`recycle` lifecycle stages)

Each named object must carry:
- `policyPackId` / `policyPackVersion`
- `inputHash` / `bundleDigest`
- signer identity + `trustAnchorLevel`
- validity interval
- verifier instructions

## Version Pinning

All artifacts SHALL declare:

- TASC spec version
- Schema version(s)
- CLI verifier version

Buyer may reject artifacts generated from deprecated or unpinned versions.

## Audit Rights

Buyer or Buyer designee may independently verify artifacts and proofs using the public verifier CLI.

## Prime Flowdown

Prime integrators SHALL flow these requirements to subcontractors whose components influence:

- Safety-relevant control paths
- Interlock authority boundaries
- Safety logging and replay evidence

## Data Rights and Confidentiality

Artifacts are safety-critical deliverables. Confidential handling applies, but vendor may not withhold artifacts needed for safety audit, incident investigation, or claims handling.
