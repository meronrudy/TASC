# Reference Interlock Scope

This directory defines external interlock integration guidance for hardware/software gates.

## Contract
- Default-closed gate semantics are mandatory.
- Only safety-kernel-sourced authority is accepted.
- Fault-latched shutdown semantics must remain active until explicit reset.

## Runtime
- `runtime.py` provides `InterlockGate.evaluate(...)` with:
- `default-closed` behavior
- authority exclusivity enforcement (`safety_kernel` only)
- heartbeat threshold enforcement
- latched shutdown state requiring explicit reset
