# ASTM / EASA / FAA Regulatory Alignment Map

This table provides clause-level alignment anchors for procurement and underwriting evidence review.

| External anchor | TASC requirement | Verifier check / artifact | Trace IDs |
| --- | --- | --- | --- |
| EASA Part-IS (compliance in force 2025-10-16; authority requirements 2026-02-22) | Information-security evidence and auditable controls | `SignedOperationalLog`, transparency proofs, badge status | MAP-REG-001 |
| EU AI Act Article 12/19 | Automatic logging and six-month minimum retention | CHK_RETENTION_EU_MINIMUM + log schema checks | MAP-REG-002 |
| EU AI Act Article 73 | Incident timeline fields 15/2/10 days | CHK_INCIDENT_WINDOWS_EU | MAP-REG-003 |
| FAA AC 20-115D (DO-178C context) | Objective-linked lifecycle evidence | `evidence/manifests/*`, tracecheck/hashboard outputs | MAP-REG-004 |
| FAA AC 20-174 (ARP4754A context) | System architecture verification evidence | Topology assertions A001/A002 + interlock checks | MAP-REG-005 |
| FAA AC 20-140C / RTCA security references | Cyber and provenance evidence continuity | TA2 attestation + dual transparency checks | MAP-REG-006 |

## Release Baseline
- Mandatory policy: `eu-north-star`
- Mandatory attestation floor: `TA2`
- Mandatory transparency logs: `rekor`, `mirror`
- Mandatory profile coverage: `uas-small`, `fixed-wing`, `hybrid-vtol`
