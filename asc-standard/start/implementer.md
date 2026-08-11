# Implementer Lane

TASC for an implementer means extending or embedding the reference runtime surface without binding to governance internals.

Start with:
- `reference/kernel/`
- `reference/adapters/`
- `reference/interlock/`
- `reference/supervisor/`

Runnable example:

```bash
./tasc doctor --operation verify
./tasc verify examples/minimal-replay
```

Architecture view:
- kernel enforces deterministic safety behavior
- adapter, interlock, and supervisor model the outer control boundary
- verifier and pack tooling are downstream acceptance surfaces

Canonical command sequence:
- `cargo test --manifest-path reference/kernel/Cargo.toml --workspace`
- `./tasc verify examples/minimal-replay`
- `./tasc diff examples/minimal-local/expected-report.json examples/minimal-replay/expected-report.json`

What not to read yet:
- incident runbooks
- risk acceptance records
- release close checklist

Next safe step:
- keep your implementation pinned to one profile bundle and one contract version at a time.
