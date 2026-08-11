# Minimal Replay

This example keeps the same product surface but emphasizes replay and report reproducibility.

Use it for:
- `doctor` on the replay path
- `verify` against a replay-oriented golden report
- `diff` between two report shapes

Commands:

```bash
./tasc doctor --operation verify
./tasc verify examples/minimal-replay
./tasc diff examples/minimal-local/expected-report.json examples/minimal-replay/expected-report.json
```

What not to read yet:
- procurement templates
- operator runbooks
- governance records

Next safe step:
- connect replay validation to your own generated bundle and pin the same profile bundle in CI.
