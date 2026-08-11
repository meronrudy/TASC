# Operator Lane

TASC for an operator means predictable diagnostics, support bundles, and a clear separation between dev-safe and production-trust flows.

Start with:
- `./tasc doctor`
- `./tasc support-bundle`
- `docs/runbooks/README.md`

Runnable example:

```bash
./tasc doctor --operation pack --profile-bundle staging-signed@1.0.0
./tasc support-bundle examples/minimal-signed
./tasc redact-support-bundle .tasc/support-bundle
```

Architecture view:
- wrapper CLI resolves config and trust mode
- operator workflows consume reports, support bundles, and runbooks
- production release steps remain in the deeper handbook/runbook layer

Canonical command sequence:
- `./tasc doctor --operation pack`
- `./tasc support-bundle examples/minimal-signed`
- `./tasc redact-support-bundle .tasc/support-bundle`

What not to read yet:
- schema internals
- theorem indexes
- procurement templates

Next safe step:
- externalize production trust assets before using `prod-hsm`.
