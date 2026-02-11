# Reference Supervisor Scope

Supervisor orchestration coordinates runtime mode transitions and incident pack workflows.

## Contract
- Supervisor may request modes; kernel/interlock remains final authority.
- Supervisor must persist incident metadata compatible with `incident-*` schemas.

## Runtime
- `runtime.py` implements:
- incident initial template generation (`tasc-incident-initial-v0.1`)
- auditor-ready incident pack generation (`tasc-incident-pack-v0.1`)
- mandatory evidence reference carry-through for regulator/insurer timelines
