# Integrator Lane

TASC for an integrator means one thing: integrate to a stable interface, not to repo internals.

Start with:
- `spec/interfaces/api.openapi.yaml`
- `reference/contracts/interfaces.v1.yaml`

Runnable example:

```bash
./tasc doctor --operation verify
./tasc verify examples/minimal-local
```

Architecture view:
- external systems talk to the API and contract surface
- reference components are examples, not required dependencies
- governance and release tooling are downstream of integration

Canonical command sequence:
- `./tasc doctor --operation verify`
- `./tasc verify examples/minimal-local`
- `./tasc explain last`

What not to read yet:
- runbooks
- release criteria
- proofs
- safety-case mappings

Next safe step:
- pin `api.openapi.yaml` and `interfaces.v1.yaml` versions in your own client build.
