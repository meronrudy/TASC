# TASC MSA Language v0.1

## Safety Assurance Artifacts

Vendor shall deliver a TASC Assurance Pack for every release and safety-relevant configuration.

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
