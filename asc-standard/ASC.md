# Avionics Safety Contract (ASC) Standard

## Canonical Sources

The canonical specification for ASC is **this document (`ASC.md`) together with all normative YAML under `spec/`**. Implementations, generated code, and documentation are derivative artifacts and MUST NOT conflict with canonical sources.

## Core Tuple

ASC evaluates each control tick using the tuple `(S, F, E, G, I)`:

- `S` — State validity and frame integrity checks.
- `F` — Flow and actuator feasibility checks.
- `E` — Energy budget and reserve checks.
- `G` — Temporal guarantees and deadline compliance.
- `I` — Safety invariants and barrier conditions.

## Change Classes

- **Class A**: Normative semantic change impacting safety decisions, precedence, or admissible behavior.
- **Class B**: Backward-compatible extension adding optional fields/profiles/tests.
- **Class C**: Editorial/documentation/tooling updates with no semantic impact.
