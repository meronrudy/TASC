# ARP4754A System Development Assurance Map

This map ties ASC architecture evidence to ARP4754A development-assurance expectations (via FAA AC 20-174 context).

| ARP4754A concept | ASC/TASC artifact(s) | Evidence path | Trace IDs |
| --- | --- | --- | --- |
| Aircraft/system requirements capture | `spec/profiles/*.yaml`, `spec/asc/state-se3.yaml`, `spec/asc/invariants-rcbf.yaml` | Profile-specific spec generation and hash | MAP-4754A-001 |
| Architecture allocation and interfaces | `evidenceMap.topology.*`, `spec/interfaces/*` | CHK_TOPOLOGY_INTERLOCK_MEDIATION, CHK_TOPOLOGY_AUTHORITY_EXCLUSIVITY | MAP-4754A-002 |
| Integration verification | `conformance/profiles/*-suite.yaml`, `conformance/suites/*/suite.yaml` | Conformance index built from verifier outputs | MAP-4754A-003 |
| Development assurance level discipline | `spec/tasc/policy-eu-north-star.yaml`, `spec/tasc/trust-anchor-levels.yaml` | TA2 enforcement and dual transparency checks | MAP-4754A-004 |
| Change impact and lifecycle continuity | `governance/CHANGE_CONTROL.md`, release gates in `.github/workflows/*.yml` | Hard-fail CI and release preflight checks | MAP-4754A-005 |

## Evidence Hooks
- `evidence/manifests/tasc-conformance-uas-small.json`
- `evidence/manifests/tasc-conformance-fixed-wing.json`
- `evidence/manifests/tasc-conformance-hybrid-vtol.json`
