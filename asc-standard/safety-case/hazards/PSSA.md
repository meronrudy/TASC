# Preliminary System Safety Assessment

## Safety allocation

| Safety objective | Allocated element | Verification evidence |
|---|---|---|
| Reject invalid frame transitions | ASC kernel `S` check | `TST-INV-001`, kernel conformance logs |
| Bound commanded rates and climb | ASC kernel `F` check + constrain | kernel model test outputs |
| Enforce SOC reserve floor | ASC kernel `E` check | `TST-GUA-001` + energy traces |
| Enforce temporal guarantees | ASC kernel `G` monitor | `TST-GUA-001`, replay parity |
| Enforce control invariants | ASC kernel `I` check | `TST-INV-001` shutdown evidence |

## Failure containment

- Single tick decision is deterministic and fully logged via hash-chained records.
- Severe checks escalate via verdict precedence (`Shutdown > Override > Hold > Clamp > Allow`).
- Contract fingerprint is attached to each output for reproducibility and replay attribution.

## Open actions

1. Add HIL evidence for shutdown latency objective.
2. Add profile-specific fault-injection vectors for hybrid VTOL mode.
