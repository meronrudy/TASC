# Minimal Local

This example is the first-run path for `./tasc`.

Use it for:
- offline-safe `doctor`
- report-shape verification against a known-good verifier report
- `explain`
- support bundle generation
- smoke-checking the `pack` path up to current trust-artifact freshness limits

Commands:

```bash
./tasc doctor --operation verify
./tasc verify examples/minimal-local
./tasc explain last
./tasc support-bundle examples/minimal-local
```

What this example intentionally does not prove yet:
- fresh live transparency proofs
- HSM-backed signing
- release-grade provenance packaging
- successful `pack` with an expired repo-shipped revocation snapshot

Next safe step:
- Move to `examples/minimal-signed` once you need pack/sign semantics.
