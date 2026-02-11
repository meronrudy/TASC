# Functional Hazard Assessment

## Scope

Initial FHA for ASC kernel decisioning path `(S,F,E,G,I)` in reference integration.

## Hazard register

| Hazard ID | Description | Severity | Mitigation in ASC | Residual risk |
|---|---|---|---|---|
| HZ-ASC-001 | Invalid frame causes unsafe control output | Catastrophic | `StateInvalidFrame` -> `Shutdown` verdict | Low |
| HZ-ASC-002 | Energy depletion during mission segment | Hazardous | `EnergyBudgetExceeded` -> `Hold` / mission abort path | Medium |
| HZ-ASC-003 | Temporal overrun misses control deadlines | Major | `TemporalGuaranteeViolation` + `DeadlineMiss` -> `Override` | Medium |
| HZ-ASC-004 | Invariant breach (altitude/bank) | Catastrophic | `InvariantViolation` -> `Shutdown` with logged evidence | Low |

## Assumptions

- Platform monotonic clock is available.
- Flight computer can actuate shutdown command deterministically.
- Input freshness telemetry (`input_age_ms`) is trustworthy within calibrated bounds.
