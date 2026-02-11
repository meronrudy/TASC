# REQ-LL-001: Temporal Enforcement

The kernel shall emit deterministic `TemporalGuaranteeViolation` and `DeadlineMiss` reason codes when timing constraints are breached.

Evidence:
- `reference/kernel/crates/asc-conformance-kernel/tests/determinism.rs`
- CHK_REPLAY_COMPLETENESS
