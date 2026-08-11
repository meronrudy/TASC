# Auditor Lane

TASC for an auditor means reading stable reports, versioned profile bundles, and reproducible evidence contracts without reverse-engineering the implementation tree.

Start with:
- `docs/handbook/FAILURE_SCHEMA.md`
- `docs/handbook/CONFIG_CONTRACT.md`
- `RELEASE_CRITERIA.md`

Runnable example:

```bash
./tasc verify examples/minimal-local --format json
./tasc explain last --format sarif
```

Architecture view:
- versioned profile bundle + versioned contract surface drive the report shape
- support bundles capture the environment and config provenance
- release and governance docs remain the higher-assurance layer

Canonical command sequence:
- `./tasc verify examples/minimal-local --format json`
- `./tasc explain last --format sarif`
- `./tasc support-bundle examples/minimal-local`

What not to read yet:
- Rust crate internals
- runtime adapter code
- full conformance vector forests

Next safe step:
- review profile bundle ids and contract versions before evaluating report verdicts.
