# Bus Mapping

This mapping defines how ASC kernel input/output fields are transported across an avionics integration boundary.

## Inbound mapping (`KernelInput`)

| ASC field | Bus topic / signal | Units | Notes |
|---|---|---|---|
| `tick.seq` | `fc.tick.seq` | count | Monotonic sequence from flight computer. |
| `tick.ts_ms` | `time.monotonic_ms` | ms | Monotonic, synchronized by platform clock. |
| `state.frame` | `nav.frame_id` | enum | Must equal `NED` for current profile. |
| `state.position_m` | `nav.position_ned` | m | 3-vector `[n,e,d]`. |
| `state.velocity_mps` | `nav.speed` | m/s | Scalar speed estimate. |
| `state.bank_deg` | `nav.bank_angle` | deg | Signed bank angle. |
| `state.soc_percent` | `power.soc` | % | Battery state of charge. |
| `state.input_age_ms` | `fusion.input_age_ms` | ms | Age of fused state input. |
| `intent.desired_rates_dps` | `guidance.rates_cmd` | deg/s | 3-vector body rates. |
| `intent.desired_climb_mps` | `guidance.climb_cmd` | m/s | Signed climb command. |

## Outbound mapping (`KernelOutput`)

| ASC field | Bus topic / signal | Notes |
|---|---|---|
| `verdict` | `asc.verdict` | One of `Allow`, `Clamp`, `Hold`, `Override`, `Shutdown`. |
| `reasons[]` | `asc.reasons` | Reason code list for auditability. |
| `command.applied_rates_dps` | `actuation.rates_cmd_safe` | Safety-constrained rates. |
| `command.applied_climb_mps` | `actuation.climb_cmd_safe` | Safety-constrained climb. |
| `command.shutdown` | `actuation.shutdown` | Hard shutdown latch if true. |
| `contract_fingerprint` | `asc.contract_fp` | 64-char SHA-256 hex digest. |

## Determinism notes

- Input capture must be ordered by `tick.seq`.
- Transport retries MUST NOT reorder packets for the same `tick.seq`.
- Any missing inbound frame should be surfaced as stale input in the next tick.
