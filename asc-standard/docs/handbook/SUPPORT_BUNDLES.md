# Support Bundles

Use support bundles to move diagnosis from “cannot reproduce” to “same config, same report, same environment”.

## Commands

```bash
./tasc support-bundle examples/minimal-local
./tasc redact-support-bundle .tasc/support-bundle
```

## Included Data

- CLI version
- resolved config
- profile bundle id and version
- doctor output
- verifier report if present
- normalized explain output if present
- bundle manifest

## Redaction Policy

`redact-support-bundle` replaces sensitive values with metadata-only placeholders.

Current redaction targets:
- signatures
- nonces
- PIN-like fields
- evidence signature material
