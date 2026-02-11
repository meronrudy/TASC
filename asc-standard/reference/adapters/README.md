# Reference Adapters Scope

This directory contains interface adapters between platform-specific buses and ASC kernel inputs.

## Contract
- Input normalization must produce `asc_types::model::KernelInput`.
- Adapter outputs must not bypass `safety_kernel -> interlock_gate` authority path.
- Versioned contract baseline: `1.0.0`.

## Runtime
- `runtime.py` exposes `normalize_to_kernel_intent(payload)` for profile-normalized adapter output.
- The adapter rejects any `authoritySource` other than `safety_kernel`.

## Quick check
```bash
python3 -c 'from reference.adapters.runtime import normalize_to_kernel_intent; print(normalize_to_kernel_intent({"profile":"uas-small","missionId":"m1","modeRequest":"AUTO","commandVector":[0.1,0.2,0.3]}))'
```
